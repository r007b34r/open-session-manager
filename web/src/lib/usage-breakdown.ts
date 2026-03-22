import type {
  ConfigRiskRecord,
  CostSource,
  SessionDetailRecord,
  SessionUsageRecord
} from "./api";

export type ModelBreakdownEntry = {
  label: string;
  sessionCount: number;
  totalTokens: number;
  costUsd?: number;
  costSource: CostSource;
};

export type ProviderBreakdownEntry = {
  label: string;
  configCount: number;
  assistantCount: number;
  proxyCount: number;
};

type UsageAggregateState = {
  sessionCount: number;
  totalTokens: number;
  costUsd: number;
  hasKnownCost: boolean;
  hasUnknownCost: boolean;
  hasReported: boolean;
  hasEstimated: boolean;
};

export function buildModelBreakdown(sessions: SessionDetailRecord[]): ModelBreakdownEntry[] {
  const groups = new Map<string, UsageAggregateState>();

  for (const session of sessions) {
    if (!session.usage) {
      continue;
    }

    const label = session.usage.model?.trim() || "Unknown";
    const entry = groups.get(label) ?? createUsageAggregateState();
    accumulateUsage(entry, session.usage);
    groups.set(label, entry);
  }

  return [...groups.entries()]
    .map(([label, entry]) => {
      const cost = resolveAggregateCost(entry);
      return {
        label,
        sessionCount: entry.sessionCount,
        totalTokens: entry.totalTokens,
        costUsd: cost.costUsd,
        costSource: cost.costSource
      };
    })
    .sort((left, right) => {
      const tokenDelta = right.totalTokens - left.totalTokens;
      if (tokenDelta !== 0) {
        return tokenDelta;
      }

      return left.label.localeCompare(right.label);
    });
}

export function buildProviderBreakdown(configs: ConfigRiskRecord[]): ProviderBreakdownEntry[] {
  const groups = new Map<
    string,
    { configCount: number; assistants: Set<string>; proxyCount: number }
  >();

  for (const config of configs) {
    const label = config.provider.trim() || "Unknown";
    const entry = groups.get(label) ?? {
      configCount: 0,
      assistants: new Set<string>(),
      proxyCount: 0
    };

    entry.configCount += 1;
    entry.assistants.add(config.assistant);
    if (config.officialOrProxy.trim().toLowerCase() !== "official") {
      entry.proxyCount += 1;
    }

    groups.set(label, entry);
  }

  return [...groups.entries()]
    .map(([label, entry]) => ({
      label,
      configCount: entry.configCount,
      assistantCount: entry.assistants.size,
      proxyCount: entry.proxyCount
    }))
    .sort((left, right) => {
      const configDelta = right.configCount - left.configCount;
      if (configDelta !== 0) {
        return configDelta;
      }

      return left.label.localeCompare(right.label);
    });
}

function createUsageAggregateState(): UsageAggregateState {
  return {
    sessionCount: 0,
    totalTokens: 0,
    costUsd: 0,
    hasKnownCost: false,
    hasUnknownCost: false,
    hasReported: false,
    hasEstimated: false
  };
}

function accumulateUsage(state: UsageAggregateState, usage: SessionUsageRecord) {
  state.sessionCount += 1;
  state.totalTokens += usage.totalTokens;

  if (typeof usage.costUsd === "number") {
    state.hasKnownCost = true;
    state.costUsd = roundCost(state.costUsd + usage.costUsd);
  } else {
    state.hasUnknownCost = true;
  }

  switch (usage.costSource) {
    case "reported":
      state.hasReported = true;
      break;
    case "estimated":
      state.hasEstimated = true;
      break;
    case "mixed":
      state.hasReported = true;
      state.hasEstimated = true;
      break;
    default:
      state.hasUnknownCost = true;
      break;
  }
}

function resolveAggregateCost(state: UsageAggregateState) {
  if (state.hasUnknownCost || !state.hasKnownCost) {
    return {
      costUsd: undefined,
      costSource: "unknown" as const
    };
  }

  if (state.hasReported && state.hasEstimated) {
    return {
      costUsd: roundCost(state.costUsd),
      costSource: "mixed" as const
    };
  }

  if (state.hasEstimated) {
    return {
      costUsd: roundCost(state.costUsd),
      costSource: "estimated" as const
    };
  }

  return {
    costUsd: roundCost(state.costUsd),
    costSource: "reported" as const
  };
}

function roundCost(value: number) {
  return Math.round(value * 100000) / 100000;
}
