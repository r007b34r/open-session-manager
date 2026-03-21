export function normalizeFixtureSnapshot(snapshot) {
  return visit(snapshot, "$");
}

export function diffFixtureSnapshots(expected, actual) {
  const differences = [];
  diffValue(expected, actual, "$", differences);
  return differences;
}

function visit(value, currentPath) {
  if (Array.isArray(value)) {
    return value.map((entry, index) => visit(entry, `${currentPath}[${index}]`));
  }

  if (isRecord(value)) {
    const normalized = {};
    for (const key of Object.keys(value)) {
      if (currentPath === "$.runtime" && RUNTIME_KEYS.has(key)) {
        normalized[key] = "<runtime>";
        continue;
      }

      normalized[key] = visit(value[key], `${currentPath}.${key}`);
    }
    return normalized;
  }

  return value;
}

function diffValue(expected, actual, currentPath, differences) {
  if (Array.isArray(expected) && Array.isArray(actual)) {
    const length = Math.max(expected.length, actual.length);
    for (let index = 0; index < length; index += 1) {
      diffValue(expected[index], actual[index], `${currentPath}[${index}]`, differences);
    }
    return;
  }

  if (isRecord(expected) && isRecord(actual)) {
    const keys = new Set([...Object.keys(expected), ...Object.keys(actual)]);
    for (const key of [...keys].sort()) {
      diffValue(expected[key], actual[key], `${currentPath}.${key}`, differences);
    }
    return;
  }

  if (Object.is(expected, actual)) {
    return;
  }

  differences.push({
    path: currentPath,
    expected,
    actual
  });
}

function isRecord(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

const RUNTIME_KEYS = new Set([
  "auditDbPath",
  "exportRoot",
  "defaultExportRoot",
  "quarantineRoot",
  "preferencesPath"
]);
