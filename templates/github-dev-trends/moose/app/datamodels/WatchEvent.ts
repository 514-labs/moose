import { Key } from "@514labs/moose-lib";

export interface WatchEvent {
  eventId: Key<string>;
  createdAt: Date;
  actorLogin: string;
  actorId: number;
  actorUrl: string;
  actorAvatarUrl: string;
  repoName: string;
  repoUrl: string;
  repoId: number;
}

export interface WatchEventWithRepo extends WatchEvent {
  repoDescription?: string;
  repoTopics?: string[];
  repoLanguage?: string;
  repoStars?: number;
  repoForks?: number;
  repoWatchers?: number;
  repoOpenIssues?: number;
  repoCreatedAt?: Date;
  repoOwnerLogin?: string;
  repoOwnerId?: number;
  repoOwnerUrl?: string;
  repoOwnerAvatarUrl?: string;
  repoOwnerType?: string;
  repoOrgId?: number;
  repoOrgUrl?: string;
  repoOrgLogin?: string;
  repoHomepage?: string;
}
