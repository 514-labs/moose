// This file was auto-generated by the framework. You can add data models or change the existing ones
type Key<T extends string | number> = T;

interface PageViewEvent {
  eventId: Key<string>;
  timestamp: Date;
  session_id: string;
  user_agent: string;
  locale: string;
  location: string;
  href: string;
  pathname: string;
  referrer: string;
}