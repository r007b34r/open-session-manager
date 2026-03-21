import { ConfigRiskPanel } from "../components/config-risk-panel";
import type { ConfigRiskRecord, ConfigWritebackInput } from "../lib/api";

type ConfigsRouteProps = {
  configs: ConfigRiskRecord[];
  canEditConfigs?: boolean;
  onSaveConfig?: (input: ConfigWritebackInput) => void;
};

export function ConfigsRoute({
  configs,
  canEditConfigs = false,
  onSaveConfig
}: ConfigsRouteProps) {
  return (
    <ConfigRiskPanel
      canEditConfigs={canEditConfigs}
      configs={configs}
      onSaveConfig={onSaveConfig}
    />
  );
}
