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
  focus?: SessionSearchFocus;
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
  tokens: string[];
  transcriptIndex?: number;
};

export type SessionSearchFocus = {
  kind: "transcript";
  highlightIndex: number;
  terms: string[];
};

type IndexedSession = {
  session: SessionDetailRecord;
  fields: SearchField[];
  originalIndex: number;
};

type QueryStats = {
  documentCount: number;
  averageFieldLengths: Record<SessionSearchReason, number>;
  documentFrequency: Map<string, number>;
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

const BM25_K1 = 1.2;
const BM25_B = 0.75;
const EXACT_TITLE_PHRASE_BONUS = FIELD_WEIGHTS.title * 3;

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

  const indexedSessions = sessions.map((session, index) => ({
    session,
    fields: buildFields(session),
    originalIndex: index
  }));
  const stats = buildQueryStats(indexedSessions, terms);

  return indexedSessions
    .map(({ session, fields, originalIndex }) =>
      scoreSession(session, fields, terms, stats, originalIndex)
    )
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
  fields: SearchField[],
  terms: SearchTerm[],
  stats: QueryStats,
  originalIndex: number
): (SessionSearchResult & { originalIndex: number }) | null {
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
      score += scoreMatch(field, term, stats);
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
    focus: buildSearchFocus(snippetField, terms),
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

  for (const [index, highlight] of session.transcriptHighlights.entries()) {
    fields.push(buildField("transcript", highlight.content, { transcriptIndex: index }));
  }

  for (const todo of session.todoItems) {
    fields.push(buildField("todo", todo.content));
  }

  return fields.filter((field) => field.text.length > 0);
}

function buildField(
  reason: SessionSearchReason,
  originalText: string,
  options: Pick<SearchField, "transcriptIndex"> = {}
): SearchField {
  const text = normalizeSearchText(originalText);
  return {
    reason,
    originalText,
    text,
    weight: FIELD_WEIGHTS[reason],
    tokens: tokenize(text),
    ...options
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

function scoreMatch(field: SearchField, term: SearchTerm, stats: QueryStats) {
  const termFrequency = countTermFrequency(field, term);
  if (termFrequency === 0) {
    return 0;
  }

  const documentFrequency = Math.max(stats.documentFrequency.get(buildTermKey(term)) ?? 0, 1);
  const averageFieldLength = stats.averageFieldLengths[field.reason] || 1;
  const lengthRatio = field.tokens.length / averageFieldLength;
  const inverseDocumentFrequency = Math.log(
    1 + (stats.documentCount - documentFrequency + 0.5) / (documentFrequency + 0.5)
  );
  const normalizedFrequency =
    (termFrequency * (BM25_K1 + 1)) /
    (termFrequency + BM25_K1 * (1 - BM25_B + BM25_B * lengthRatio));
  const fieldScore = (field.weight / 10) * inverseDocumentFrequency * normalizedFrequency * 100;
  const phraseBonus = term.phrase ? field.weight * 0.2 : 0;
  const exactTitleBonus =
    term.phrase && field.reason === "title" && field.text === term.value
      ? EXACT_TITLE_PHRASE_BONUS
      : 0;

  return fieldScore + phraseBonus + exactTitleBonus;
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

function countTermFrequency(field: SearchField, term: SearchTerm) {
  if (term.phrase) {
    return countOccurrences(field.text, term.value);
  }

  return field.tokens.filter((token) => token.startsWith(term.value)).length;
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

function buildSearchFocus(
  field: SearchField | undefined,
  terms: SearchTerm[]
): SessionSearchFocus | undefined {
  if (!field || field.reason !== "transcript" || typeof field.transcriptIndex !== "number") {
    return undefined;
  }

  const matchedTerms = terms
    .filter((term) => matchesTerm(field.text, term))
    .map((term) => term.value)
    .filter((value, index, values) => values.indexOf(value) === index);

  if (matchedTerms.length === 0) {
    return undefined;
  }

  return {
    kind: "transcript",
    highlightIndex: field.transcriptIndex,
    terms: matchedTerms
  };
}

function tokenize(text: string) {
  return text.split(/\s+/).filter(Boolean);
}

function buildQueryStats(
  sessions: IndexedSession[],
  terms: SearchTerm[]
): QueryStats {
  const totals = new Map<SessionSearchReason, { totalLength: number; count: number }>();
  const documentFrequency = new Map<string, number>();

  for (const indexed of sessions) {
    for (const field of indexed.fields) {
      const current = totals.get(field.reason) ?? { totalLength: 0, count: 0 };
      current.totalLength += Math.max(field.tokens.length, 1);
      current.count += 1;
      totals.set(field.reason, current);
    }

    for (const term of terms) {
      if (!indexed.fields.some((field) => matchesTerm(field.text, term))) {
        continue;
      }

      const termKey = buildTermKey(term);
      documentFrequency.set(termKey, (documentFrequency.get(termKey) ?? 0) + 1);
    }
  }

  const averageFieldLengths = FIELD_ORDER.reduce((accumulator, reason) => {
    const total = totals.get(reason);
    accumulator[reason] = total ? total.totalLength / total.count : 1;
    return accumulator;
  }, {} as Record<SessionSearchReason, number>);

  return {
    documentCount: Math.max(sessions.length, 1),
    averageFieldLengths,
    documentFrequency
  };
}

function buildTermKey(term: SearchTerm) {
  return `${term.phrase ? "phrase" : "term"}:${term.value}`;
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
