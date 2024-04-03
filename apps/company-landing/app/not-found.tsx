import { sendServerEvent } from "event-capture/server-event";
import Link from "next/link";

export default function NotFound() {
  sendServerEvent("not-found-404", {});
  return (
    <div>
      <h2>Not Found</h2>
      <p>Page Not found</p>
      <Link href="/">Return Home</Link>
    </div>
  );
}
