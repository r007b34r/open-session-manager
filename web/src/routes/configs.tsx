import { ConfigRiskPanel } from "../components/config-risk-panel";
import type { ConfigRiskRecord } from "../lib/api";

type ConfigsRouteProps = {
  configs: ConfigRiskRecord[];
};

export function ConfigsRoute({ configs }: ConfigsRouteProps) {
  return <ConfigRiskPanel configs={configs} />;
}
