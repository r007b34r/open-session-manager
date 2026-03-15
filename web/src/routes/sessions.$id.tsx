import { SessionDetail } from "../components/session-detail";
import type { SessionDetailRecord } from "../lib/api";

type SessionDetailRouteProps = {
  session?: SessionDetailRecord;
};

export function SessionDetailRoute({ session }: SessionDetailRouteProps) {
  return <SessionDetail session={session} />;
}
