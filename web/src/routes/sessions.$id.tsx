import { SessionDetail } from "../components/session-detail";
import type { SessionDetailRecord } from "../lib/api";

type SessionDetailRouteProps = {
  session?: SessionDetailRecord;
  canSoftDelete?: boolean;
  onExportMarkdown?: (sessionId: string) => void;
  onSoftDelete?: (sessionId: string) => void;
};

export function SessionDetailRoute({
  session,
  canSoftDelete,
  onExportMarkdown,
  onSoftDelete
}: SessionDetailRouteProps) {
  return (
    <SessionDetail
      canSoftDelete={canSoftDelete}
      onExportMarkdown={onExportMarkdown}
      onSoftDelete={onSoftDelete}
      session={session}
    />
  );
}
