import { SessionDetail } from "../components/session-detail";
import type { SessionDetailRecord } from "../lib/api";

type SessionDetailRouteProps = {
  session?: SessionDetailRecord;
  onExportMarkdown?: (sessionId: string) => void;
  onSoftDelete?: (sessionId: string) => void;
};

export function SessionDetailRoute({
  session,
  onExportMarkdown,
  onSoftDelete
}: SessionDetailRouteProps) {
  return (
    <SessionDetail
      onExportMarkdown={onExportMarkdown}
      onSoftDelete={onSoftDelete}
      session={session}
    />
  );
}
