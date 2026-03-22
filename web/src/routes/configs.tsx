import { ConfigRiskPanel } from "../components/config-risk-panel";
import type {
  ConfigRiskRecord,
  ConfigWritebackInput,
  LocalAuditEventInput
} from "../lib/api";

type ConfigsRouteProps = {
  configs: ConfigRiskRecord[];
  canEditConfigs?: boolean;
  onSaveConfig?: (input: ConfigWritebackInput) => void;
  onAuditEvent?: (input: LocalAuditEventInput) => void;
};

export function ConfigsRoute({
  configs,
  canEditConfigs = false,
  onSaveConfig,
  onAuditEvent
}: ConfigsRouteProps) {
  return (
    <ConfigRiskPanel
      onAuditEvent={onAuditEvent}
      canEditConfigs={canEditConfigs}
      configs={configs}
      onSaveConfig={onSaveConfig}
    />
  );
}
