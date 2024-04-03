import { sendServerEvent } from "event-capture/server-event";

export default function NotFound() {
  sendServerEvent("not-found-404", {});
  return (
    <div>
      <h2>Not Found</h2>
      <p>Page Not found</p>
      <a href="/">Return Home</a>
    </div>
  );
}
