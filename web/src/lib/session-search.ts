import type { SessionDetailRecord } from "./api";

export type SessionSearchReason =
  | "title"
  | "assistant"
  | "environment"
  | "summary"
  | "project"
  | "source"
  | "tag"
  | "risk"
  | "artifact"
  | "transcript"
  | "todo";

export type SessionSearchResult = {
  session: SessionDetailRecord;
  score: number;
  snippet?: string;
  matchReasons: SessionSearchReason[];
};

type SearchTerm = {
  value: string;
  phrase: boolean;
};

type SearchField = {
  reason: SessionSearchReason;
  text: string;
  originalText: string;
  weight: number;
};

const FIELD_WEIGHTS: Readonly<Record<SessionSearchReason, number>> = {
  title: 120,
  assistant: 30,
  environment: 18,
  summary: 70,
  project: 42,
  source: 12,
  tag: 34,
  risk: 22,
  artifact: 48,
  transcript: 52,
  todo: 58
};

const FIELD_ORDER: readonly SessionSearchReason[] = [
  "title",
  "summary",
  "project",
  "artifact",
  "transcript",
  "todo",
  "assistant",
  "environment",
  "tag",
  "risk",
  "source"
];

export function searchSessions(
  sessions: SessionDetailRecord[],
  query: string
): SessionSearchResult[] {
  const terms = parseSearchTerms(query);
  if (terms.length === 0) {
    return sessions.map((session) => ({
      session,
      score: 0,
      matchReasons: []
    }));
  }

  return sessions
    .map((session, index) => scoreSession(session, terms, index))
    .filter((result): result is SessionSearchResult & { originalIndex: number } => {
      return result !== null;
    })
    .sort((left, right) => {
      const scoreDelta = right.score - left.score;
      if (scoreDelta !== 0) {
        return scoreDelta;
      }

      return left.originalIndex - right.originalIndex;
    })
    .map(({ originalIndex: _originalIndex, ...result }) => result);
}

function scoreSession(
  session: SessionDetailRecord,
  terms: SearchTerm[],
  originalIndex: number
): (SessionSearchResult & { originalIndex: number }) | null {
  const fields = buildFields(session);
  const matchedReasons = new Set<SessionSearchReason>();
  let score = 0;

  for (const term of terms) {
    let matched = false;

    for (const field of fields) {
      if (!matchesTerm(field.text, term)) {
        continue;
      }

      matched = true;
      matchedReasons.add(field.reason);
      score += scoreMatch(field, term);
    }

    if (!matched) {
      return null;
    }
  }

  const sortedReasons = [...matchedReasons].sort(compareReasons);
  const snippetField = pickSnippetField(fields, matchedReasons, terms);

  return {
    session,
    score,
    snippet: snippetField ? extractSnippet(snippetField.originalText, terms) : undefined,
    matchReasons: sortedReasons,
    originalIndex
  };
}

function buildFields(session: SessionDetailRecord): SearchField[] {
  const fields: SearchField[] = [
    buildField("title", session.title),
    buildField("assistant", session.assistant),
    buildField("environment", session.environment),
    buildField("summary", session.summary),
    buildField("project", session.projectPath),
    buildField("source", session.sourcePath)
  ];

  for (const tag of session.tags) {
    fields.push(buildField("tag", tag));
  }

  for (const risk of session.riskFlags) {
    fields.push(buildField("risk", risk));
  }

  for (const artifact of session.keyArtifacts) {
    fields.push(buildField("artifact", artifact));
  }

  for (const highlight of session.transcriptHighlights) {
    fields.push(buildField("transcript", highlight.content));
  }

  for (const todo of session.todoItems) {
    fields.push(buildField("todo", todo.content));
  }

  return fields.filter((field) => field.text.length > 0);
}

function buildField(reason: SessionSearchReason, originalText: string): SearchField {
  return {
    reason,
    originalText,
    text: normalizeSearchText(originalText),
    weight: FIELD_WEIGHTS[reason]
  };
}

function parseSearchTerms(query: string): SearchTerm[] {
  const matches = query.matchAll(/"([^"]+)"|(\S+)/g);
  const terms: SearchTerm[] = [];

  for (const match of matches) {
    const rawValue = match[1] ?? match[2] ?? "";
    const value = normalizeSearchText(rawValue);
    if (!value) {
      continue;
    }

    terms.push({
      value,
      phrase: Boolean(match[1])
    });
  }

  return terms;
}

function matchesTerm(text: string, term: SearchTerm) {
  if (!text) {
    return false;
  }

  if (text.includes(term.value)) {
    return true;
  }

  if (term.phrase) {
    return false;
  }

  return tokenize(text).some((token) => token.startsWith(term.value));
}

function scoreMatch(field: SearchField, term: SearchTerm) {
  const occurrences = countOccurrences(field.text, term.value);
  const phraseBonus = term.phrase ? 18 : 0;
  const occurrenceBonus = Math.min(occurrences, 3) * 4;
  return field.weight + phraseBonus + occurrenceBonus;
}

function countOccurrences(text: string, term: string) {
  if (!term) {
    return 0;
  }

  let count = 0;
  let cursor = 0;

  while (cursor < text.length) {
    const index = text.indexOf(term, cursor);
    if (index === -1) {
      break;
    }

    count += 1;
    cursor = index + term.length;
  }

  return count;
}

function pickSnippetField(
  fields: SearchField[],
  matchedReasons: ReadonlySet<SessionSearchReason>,
  terms: SearchTerm[]
) {
  return fields
    .filter((field) => matchedReasons.has(field.reason))
    .sort((left, right) => {
      const leftProfile = buildSnippetProfile(left, terms);
      const rightProfile = buildSnippetProfile(right, terms);

      const matchedTermDelta = rightProfile.matchedTerms - leftProfile.matchedTerms;
      if (matchedTermDelta !== 0) {
        return matchedTermDelta;
      }

      const phraseDelta = rightProfile.phraseBonus - leftProfile.phraseBonus;
      if (phraseDelta !== 0) {
        return phraseDelta;
      }

      const spanDelta = leftProfile.compactSpan - rightProfile.compactSpan;
      if (spanDelta !== 0) {
        return spanDelta;
      }

      const weightDelta = right.weight - left.weight;
      if (weightDelta !== 0) {
        return weightDelta;
      }

      return compareReasons(left.reason, right.reason);
    })[0];
}

function extractSnippet(text: string, terms: SearchTerm[]) {
  const trimmed = text.trim();
  if (!trimmed) {
    return undefined;
  }

  const lower = trimmed.toLowerCase();
  let firstMatchIndex = -1;
  let firstMatchLength = 0;

  for (const term of terms) {
    const index = lower.indexOf(term.value);
    if (index === -1) {
      continue;
    }

    if (firstMatchIndex === -1 || index < firstMatchIndex) {
      firstMatchIndex = index;
      firstMatchLength = term.value.length;
    }
  }

  if (trimmed.length <= 140 || firstMatchIndex === -1) {
    return trimmed;
  }

  const start = Math.max(0, firstMatchIndex - 36);
  const end = Math.min(trimmed.length, firstMatchIndex + firstMatchLength + 84);
  const prefix = start > 0 ? "..." : "";
  const suffix = end < trimmed.length ? "..." : "";

  return `${prefix}${trimmed.slice(start, end).trim()}${suffix}`;
}

function buildSnippetProfile(field: SearchField, terms: SearchTerm[]) {
  const indices = terms
    .map((term) => field.text.indexOf(term.value))
    .filter((index) => index >= 0)
    .sort((left, right) => left - right);
  const joinedTerms = terms.map((term) => term.value).join(" ");

  return {
    matchedTerms: indices.length,
    phraseBonus: joinedTerms && field.text.includes(joinedTerms) ? 1 : 0,
    compactSpan:
      indices.length >= 2 ? indices[indices.length - 1] - indices[0] : Number.MAX_SAFE_INTEGER
  };
}

function tokenize(text: string) {
  return text.split(/\s+/).filter(Boolean);
}

function normalizeSearchText(value: string) {
  return value
    .toLowerCase()
    .replace(/[\r\n\t]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function compareReasons(left: SessionSearchReason, right: SessionSearchReason) {
  return FIELD_ORDER.indexOf(left) - FIELD_ORDER.indexOf(right);
}
