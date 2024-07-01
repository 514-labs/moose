type Key<T extends string | number> = T;

export const MooseActivityConfig = {
  storage: {
    enabled: false,
  },
};

export const PageViewEventConfig = {
  storage: {
    enabled: false,
  },
};

export interface MooseActivity {
  id: Key<string>;
  project: string;
  activityType: string;
  sequenceId: string;
  timestamp: Date;
  cliVersion: string;
  isMooseDeveloper?: boolean;
  machineId: string;
  ip?: string;
}

export interface MooseActivityAugmented {
  id: Key<string>;
  project: string;
  activityType: string;
  sequenceId: string;
  timestamp: Date;
  cliVersion: string;
  isMooseDeveloper?: boolean;
  machineId: string;
  hashedIp?: string;
  cityName?: string;
  countryCode?: string;
  companyName?: string;
  companyType?: string;
  companyDomain?: string;
}

export interface PageViewEvent {
  eventId: Key<string>;
  timestamp: Date;
  session_id: string;
  user_agent: string;
  locale: string;
  location: string;
  href: string;
  pathname: string;
  referrer: string;
  ip?: string;
}

export interface PageViewProcessed {
  eventId: Key<string>;
  timestamp: Date;
  session_id: string;
  user_agent: string;
  locale: string;
  location: string;
  href: string;
  pathname: string;
  referrer: string;
  hashedIp?: string;
  cityName?: string;
  countryCode?: string;
  companyName?: string;
  companyType?: string;
  companyDomain?: string;
}

export interface TrackEvent extends PageViewEvent {
  action: string;
  name: string;
  subject: string;
  targetUrl?: string;
}

export interface ParsedActivity {
  eventId: Key<string>;
  timestamp: Date;
  userId: string;
  activity: string;
}

export interface UserActivity {
  eventId: Key<string>;
  timestamp: Date;
  userId: string;
  activity: string;
  description: string;
}
