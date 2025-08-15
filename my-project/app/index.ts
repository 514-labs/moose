// Welcome to your new Moose analytical backend! 🦌

// Getting Started Guide:

// 1. Data Modeling
// First, plan your data structure and create your data models
// → See: docs.fiveonefour.com/moose/building/data-modeling
//   Learn about type definitions and data validation

// 2. Set Up Ingestion
// Create ingestion pipelines to receive your data via REST APIs
// → See: docs.fiveonefour.com/moose/building/ingestion
//   Learn about IngestPipeline, data formats, and validation

// 3. Create Workflows
// Build data processing pipelines to transform and analyze your data
// → See: docs.fiveonefour.com/moose/building/workflows
//   Learn about task scheduling and data processing

// 4. Configure Consumption APIs
// Set up queries and real-time analytics for your data
// → See: docs.fiveonefour.com/moose/building/consumption-apis

// Need help? Check out the quickstart guide:
// → docs.fiveonefour.com/moose/getting-started/quickstart

import {
  IngestPipeline,
  Key,
  ClickHouseInt,
  ClickHouseDecimal,
  ClickHousePrecision,
  ClickHouseByteSize,
  ClickHouseNamedTuple,
} from "@514labs/moose-lib";
import typia from "typia";

export enum Type1 {
  "top" = 1,
  "new" = 2,
  "best" = 3,
  "ask" = 4,
  "show" = 5,
  "job" = 6,
}

export enum MergeableState {
  "unknown" = 0,
  "dirty" = 1,
  "clean" = 2,
  "unstable" = 3,
  "draft" = 4,
  "blocked" = 5,
}

export enum EventType {
  "CommitCommentEvent" = 1,
  "CreateEvent" = 2,
  "DeleteEvent" = 3,
  "ForkEvent" = 4,
  "GollumEvent" = 5,
  "IssueCommentEvent" = 6,
  "IssuesEvent" = 7,
  "MemberEvent" = 8,
  "PublicEvent" = 9,
  "PullRequestEvent" = 10,
  "PullRequestReviewCommentEvent" = 11,
  "PushEvent" = 12,
  "ReleaseEvent" = 13,
  "SponsorshipEvent" = 14,
  "WatchEvent" = 15,
  "GistEvent" = 16,
  "FollowEvent" = 17,
  "DownloadEvent" = 18,
  "PullRequestReviewEvent" = 19,
  "ForkApplyEvent" = 20,
  "Event" = 21,
  "TeamAddEvent" = 22,
}

export enum Duration {
  "unknown" = 0,
  "freehold" = 1,
  "leasehold" = 2,
}

export enum Status {
  "waiting" = 1,
  "queued" = 2,
  "in_progress" = 3,
  "completed" = 4,
}

export enum CabType {
  "yellow" = 1,
  "green" = 2,
  "uber" = 3,
}

export enum State {
  "none" = 0,
  "open" = 1,
  "closed" = 2,
}

export enum RefType {
  "none" = 0,
  "branch" = 1,
  "tag" = 2,
  "repository" = 3,
  "unknown" = 4,
}

export enum VendorId {
  // "1" = 1,
  // "2" = 2,
  // "3" = 3,
  // "4" = 4,
  "CMT" = 5,
  "VTS" = 6,
  "DDS" = 7,
  "B02512" = 10,
  "B02598" = 11,
  "B02617" = 12,
  "B02682" = 13,
  "B02764" = 14,
  // "" = 15,
}

export enum Radio {
  // "" = 0,
  "CDMA" = 1,
  "GSM" = 2,
  "LTE" = 3,
  "NR" = 4,
  "UMTS" = 5,
}

export enum Type {
  "story" = 1,
  "comment" = 2,
  "poll" = 3,
  "pollopt" = 4,
  "job" = 5,
}

export enum AuthorAssociation {
  "NONE" = 0,
  "CONTRIBUTOR" = 1,
  "OWNER" = 2,
  "COLLABORATOR" = 3,
  "MEMBER" = 4,
  "MANNEQUIN" = 5,
}

export enum Type2 {
  "other" = 0,
  "terraced" = 1,
  "semi-detached" = 2,
  "detached" = 3,
  "flat" = 4,
}

export enum ReviewState {
  "none" = 0,
  "approved" = 1,
  "changes_requested" = 2,
  "commented" = 3,
  "dismissed" = 4,
  "pending" = 5,
}

export enum PaymentType {
  "UNK" = 0,
  "CSH" = 1,
  "CRE" = 2,
  "NOC" = 3,
  "DIS" = 4,
}

export enum Action {
  "none" = 0,
  "created" = 1,
  "added" = 2,
  "edited" = 3,
  "deleted" = 4,
  "opened" = 5,
  "closed" = 6,
  "reopened" = 7,
  "assigned" = 8,
  "unassigned" = 9,
  "labeled" = 10,
  "unlabeled" = 11,
  "review_requested" = 12,
  "review_request_removed" = 13,
  "synchronize" = 14,
  "started" = 15,
  "published" = 16,
  "update" = 17,
  "create" = 18,
  "fork" = 19,
  "merged" = 20,
}

export interface actors {
  login: string;
  type: string;
  site_admin: boolean;
  name: string;
  company: string;
  blog: string;
  location: string;
  email: string;
  hireable: boolean;
  bio: string;
  twitter_username: string;
  public_repos: number & ClickHouseInt<"int64">;
  public_gists: number & ClickHouseInt<"int64">;
  followers: number & ClickHouseInt<"int64">;
  following: number & ClickHouseInt<"int64">;
  created_at: Date;
  updated_at: Date;
}

export interface benchmark_results {
  run_id: string & typia.tags.Format<"uuid">;
  query_num: number & ClickHouseInt<"uint8">;
  try_num: number & ClickHouseInt<"uint8">;
  time: string & ClickHouseDecimal<9, 3>;
}

export interface benchmark_runs {
  run_id: string & typia.tags.Format<"uuid">;
  version: string;
  test_time: Date;
  threads: string;
  fs_capacity: number & ClickHouseInt<"uint64">;
  fs_available: number & ClickHouseInt<"uint64">;
  cpu_model: string;
  cpu: string;
  df: string;
  memory: string;
  memory_total: string;
  blk: string;
  mdstat: string;
  instance: string;
}

export interface cell_towers {
  radio: Radio;
  mcc: number & ClickHouseInt<"uint16">;
  net: number & ClickHouseInt<"uint16">;
  area: number & ClickHouseInt<"uint16">;
  cell: number & ClickHouseInt<"uint64">;
  unit: number & ClickHouseInt<"int16">;
  lon: number;
  lat: number;
  range: number & ClickHouseInt<"uint32">;
  samples: number & ClickHouseInt<"uint32">;
  changeable: number & ClickHouseInt<"uint8">;
  created: Date;
  updated: Date;
  averageSignal: number & ClickHouseInt<"uint8">;
}

export interface checks {
  pull_request_number: number & ClickHouseInt<"uint32">;
  commit_sha: string;
  check_name: string;
  check_status: string;
  check_duration_ms: number & ClickHouseInt<"uint64">;
  check_start_time: Date;
  test_name: string;
  test_status: string;
  test_duration_ms: number & ClickHouseInt<"uint64">;
  report_url: string;
  pull_request_url: string;
  commit_url: string;
  task_url: string;
  base_ref: string;
  base_repo: string;
  head_ref: string;
  head_repo: string;
  test_context_raw: string;
  instance_type: string;
  instance_id: string;
}

export interface cisco_umbrella {
  date: string & typia.tags.Format<"date"> & ClickHouseByteSize<2>;
  rank: number & ClickHouseInt<"uint32">;
  domain: string;
}

export interface covid {
  date: string & typia.tags.Format<"date"> & ClickHouseByteSize<2>;
  location_key: string;
  new_confirmed: number & ClickHouseInt<"int32">;
  new_deceased: number & ClickHouseInt<"int32">;
  new_recovered: number & ClickHouseInt<"int32">;
  new_tested: number & ClickHouseInt<"int32">;
  cumulative_confirmed: number & ClickHouseInt<"int32">;
  cumulative_deceased: number & ClickHouseInt<"int32">;
  cumulative_recovered: number & ClickHouseInt<"int32">;
  cumulative_tested: number & ClickHouseInt<"int32">;
}

export interface dish {
  id: number & ClickHouseInt<"uint32">;
  name: string;
  description: string;
  menus_appeared: number & ClickHouseInt<"uint32">;
  times_appeared: number & ClickHouseInt<"int32">;
  first_appeared: number & ClickHouseInt<"uint16">;
  last_appeared: number & ClickHouseInt<"uint16">;
  lowest_price: string & ClickHouseDecimal<18, 3>;
  highest_price: string & ClickHouseDecimal<18, 3>;
}

export interface dns {
  timestamp: Date;
  domain: string;
  a: string & typia.tags.Format<"ipv4">;
  aaaa: string & typia.tags.Format<"ipv6">;
  cname: string;
}

export interface dns2 {
  timestamp: Date;
  domain: string;
  a: string & typia.tags.Format<"ipv4">;
  aaaa: string & typia.tags.Format<"ipv6">;
  cname: string;
}

export interface github_events {
  file_time: Date;
  event_type: EventType;
  actor_login: string;
  repo_name: string;
  created_at: Date;
  updated_at: Date;
  action: Action;
  comment_id: number & ClickHouseInt<"uint64">;
  body: string;
  path: string;
  position: number & ClickHouseInt<"int32">;
  line: number & ClickHouseInt<"int32">;
  ref: string;
  ref_type: RefType;
  creator_user_login: string;
  number: number & ClickHouseInt<"uint32">;
  title: string;
  labels: string[];
  state: State;
  locked: number & ClickHouseInt<"uint8">;
  assignee: string;
  assignees: string[];
  comments: number & ClickHouseInt<"uint32">;
  author_association: AuthorAssociation;
  closed_at: Date;
  merged_at: Date;
  merge_commit_sha: string;
  requested_reviewers: string[];
  requested_teams: string[];
  head_ref: string;
  head_sha: string;
  base_ref: string;
  base_sha: string;
  merged: number & ClickHouseInt<"uint8">;
  mergeable: number & ClickHouseInt<"uint8">;
  rebaseable: number & ClickHouseInt<"uint8">;
  mergeable_state: MergeableState;
  merged_by: string;
  review_comments: number & ClickHouseInt<"uint32">;
  maintainer_can_modify: number & ClickHouseInt<"uint8">;
  commits: number & ClickHouseInt<"uint32">;
  additions: number & ClickHouseInt<"uint32">;
  deletions: number & ClickHouseInt<"uint32">;
  changed_files: number & ClickHouseInt<"uint32">;
  diff_hunk: string;
  original_position: number & ClickHouseInt<"uint32">;
  commit_id: string;
  original_commit_id: string;
  push_size: number & ClickHouseInt<"uint32">;
  push_distinct_size: number & ClickHouseInt<"uint32">;
  member_login: string;
  release_tag_name: string;
  release_name: string;
  review_state: ReviewState;
}

export interface hackernews {
  id: number & ClickHouseInt<"uint32">;
  deleted: number & ClickHouseInt<"uint8">;
  type: Type;
  by: string;
  time: Date;
  text: string;
  dead: number & ClickHouseInt<"uint8">;
  parent: number & ClickHouseInt<"uint32">;
  poll: number & ClickHouseInt<"uint32">;
  kids: (number & ClickHouseInt<"uint32">)[];
  url: string;
  score: number & ClickHouseInt<"int32">;
  title: string;
  parts: (number & ClickHouseInt<"uint32">)[];
  descendants: number & ClickHouseInt<"int32">;
}

export interface hackernews_changes_items {
  update_time: Date;
  id: number & ClickHouseInt<"uint32">;
  deleted: number & ClickHouseInt<"uint8">;
  type: Type;
  by: string;
  time: Date;
  text: string;
  dead: number & ClickHouseInt<"uint8">;
  parent: number & ClickHouseInt<"uint32">;
  poll: number & ClickHouseInt<"uint32">;
  kids: (number & ClickHouseInt<"uint32">)[];
  url: string;
  score: number & ClickHouseInt<"int32">;
  title: string;
  parts: (number & ClickHouseInt<"uint32">)[];
  descendants: number & ClickHouseInt<"int32">;
}

export interface hackernews_changes_profiles {
  update_time: Date;
  about: string;
  created: Date;
  id: string;
  karma: number & ClickHouseInt<"int32">;
  submitted: (number & ClickHouseInt<"uint32">)[];
  submitted_count: number & ClickHouseInt<"uint32">;
}

export interface hackernews_history {
  update_time: Date;
  id: number & ClickHouseInt<"uint32">;
  deleted: number & ClickHouseInt<"uint8">;
  type: Type;
  by: string;
  time: Date;
  text: string;
  dead: number & ClickHouseInt<"uint8">;
  parent: number & ClickHouseInt<"uint32">;
  poll: number & ClickHouseInt<"uint32">;
  kids: (number & ClickHouseInt<"uint32">)[];
  url: string;
  score: number & ClickHouseInt<"int32">;
  title: string;
  parts: (number & ClickHouseInt<"uint32">)[];
  descendants: number & ClickHouseInt<"int32">;
}

export interface hackernews_top {
  update_time: Date;
  type: Type1;
  ids: (number & ClickHouseInt<"uint32">)[];
}

export interface hits {
  WatchID: Key<number & ClickHouseInt<"int64">>;
  JavaEnable: number & ClickHouseInt<"int16">;
  Title: string;
  GoodEvent: number & ClickHouseInt<"int16">;
  EventTime: Key<Date>;
  EventDate: Key<string & typia.tags.Format<"date"> & ClickHouseByteSize<2>>;
  CounterID: Key<number & ClickHouseInt<"int32">>;
  ClientIP: number & ClickHouseInt<"int32">;
  RegionID: number & ClickHouseInt<"int32">;
  UserID: Key<number & ClickHouseInt<"int64">>;
  CounterClass: number & ClickHouseInt<"int16">;
  OS: number & ClickHouseInt<"int16">;
  UserAgent: number & ClickHouseInt<"int16">;
  URL: string;
  Referer: string;
  IsRefresh: number & ClickHouseInt<"int16">;
  RefererCategoryID: number & ClickHouseInt<"int16">;
  RefererRegionID: number & ClickHouseInt<"int32">;
  URLCategoryID: number & ClickHouseInt<"int16">;
  URLRegionID: number & ClickHouseInt<"int32">;
  ResolutionWidth: number & ClickHouseInt<"int16">;
  ResolutionHeight: number & ClickHouseInt<"int16">;
  ResolutionDepth: number & ClickHouseInt<"int16">;
  FlashMajor: number & ClickHouseInt<"int16">;
  FlashMinor: number & ClickHouseInt<"int16">;
  FlashMinor2: string;
  NetMajor: number & ClickHouseInt<"int16">;
  NetMinor: number & ClickHouseInt<"int16">;
  UserAgentMajor: number & ClickHouseInt<"int16">;
  UserAgentMinor: string;
  CookieEnable: number & ClickHouseInt<"int16">;
  JavascriptEnable: number & ClickHouseInt<"int16">;
  IsMobile: number & ClickHouseInt<"int16">;
  MobilePhone: number & ClickHouseInt<"int16">;
  MobilePhoneModel: string;
  Params: string;
  IPNetworkID: number & ClickHouseInt<"int32">;
  TraficSourceID: number & ClickHouseInt<"int16">;
  SearchEngineID: number & ClickHouseInt<"int16">;
  SearchPhrase: string;
  AdvEngineID: number & ClickHouseInt<"int16">;
  IsArtifical: number & ClickHouseInt<"int16">;
  WindowClientWidth: number & ClickHouseInt<"int16">;
  WindowClientHeight: number & ClickHouseInt<"int16">;
  ClientTimeZone: number & ClickHouseInt<"int16">;
  ClientEventTime: Date;
  SilverlightVersion1: number & ClickHouseInt<"int16">;
  SilverlightVersion2: number & ClickHouseInt<"int16">;
  SilverlightVersion3: number & ClickHouseInt<"int32">;
  SilverlightVersion4: number & ClickHouseInt<"int16">;
  PageCharset: string;
  CodeVersion: number & ClickHouseInt<"int32">;
  IsLink: number & ClickHouseInt<"int16">;
  IsDownload: number & ClickHouseInt<"int16">;
  IsNotBounce: number & ClickHouseInt<"int16">;
  FUniqID: number & ClickHouseInt<"int64">;
  OriginalURL: string;
  HID: number & ClickHouseInt<"int32">;
  IsOldCounter: number & ClickHouseInt<"int16">;
  IsEvent: number & ClickHouseInt<"int16">;
  IsParameter: number & ClickHouseInt<"int16">;
  DontCountHits: number & ClickHouseInt<"int16">;
  WithHash: number & ClickHouseInt<"int16">;
  HitColor: string;
  LocalEventTime: Date;
  Age: number & ClickHouseInt<"int16">;
  Sex: number & ClickHouseInt<"int16">;
  Income: number & ClickHouseInt<"int16">;
  Interests: number & ClickHouseInt<"int16">;
  Robotness: number & ClickHouseInt<"int16">;
  RemoteIP: number & ClickHouseInt<"int32">;
  WindowName: number & ClickHouseInt<"int32">;
  OpenerName: number & ClickHouseInt<"int32">;
  HistoryLength: number & ClickHouseInt<"int16">;
  BrowserLanguage: string;
  BrowserCountry: string;
  SocialNetwork: string;
  SocialAction: string;
  HTTPError: number & ClickHouseInt<"int16">;
  SendTiming: number & ClickHouseInt<"int32">;
  DNSTiming: number & ClickHouseInt<"int32">;
  ConnectTiming: number & ClickHouseInt<"int32">;
  ResponseStartTiming: number & ClickHouseInt<"int32">;
  ResponseEndTiming: number & ClickHouseInt<"int32">;
  FetchTiming: number & ClickHouseInt<"int32">;
  SocialSourceNetworkID: number & ClickHouseInt<"int16">;
  SocialSourcePage: string;
  ParamPrice: number & ClickHouseInt<"int64">;
  ParamOrderID: string;
  ParamCurrency: string;
  ParamCurrencyID: number & ClickHouseInt<"int16">;
  OpenstatServiceName: string;
  OpenstatCampaignID: string;
  OpenstatAdID: string;
  OpenstatSourceID: string;
  UTMSource: string;
  UTMMedium: string;
  UTMCampaign: string;
  UTMContent: string;
  UTMTerm: string;
  FromTag: string;
  HasGCLID: number & ClickHouseInt<"int16">;
  RefererHash: number & ClickHouseInt<"int64">;
  URLHash: number & ClickHouseInt<"int64">;
  CLID: number & ClickHouseInt<"int32">;
}

export interface hits2 {
  WatchID: Key<number & ClickHouseInt<"int64">>;
  JavaEnable: number & ClickHouseInt<"int16">;
  Title: string;
  GoodEvent: number & ClickHouseInt<"int16">;
  EventTime: Key<Date>;
  EventDate: Key<string & typia.tags.Format<"date"> & ClickHouseByteSize<2>>;
  CounterID: Key<number & ClickHouseInt<"int32">>;
  ClientIP: number & ClickHouseInt<"int32">;
  RegionID: number & ClickHouseInt<"int32">;
  UserID: Key<number & ClickHouseInt<"int64">>;
  CounterClass: number & ClickHouseInt<"int16">;
  OS: number & ClickHouseInt<"int16">;
  UserAgent: number & ClickHouseInt<"int16">;
  URL: string;
  Referer: string;
  IsRefresh: number & ClickHouseInt<"int16">;
  RefererCategoryID: number & ClickHouseInt<"int16">;
  RefererRegionID: number & ClickHouseInt<"int32">;
  URLCategoryID: number & ClickHouseInt<"int16">;
  URLRegionID: number & ClickHouseInt<"int32">;
  ResolutionWidth: number & ClickHouseInt<"int16">;
  ResolutionHeight: number & ClickHouseInt<"int16">;
  ResolutionDepth: number & ClickHouseInt<"int16">;
  FlashMajor: number & ClickHouseInt<"int16">;
  FlashMinor: number & ClickHouseInt<"int16">;
  FlashMinor2: string;
  NetMajor: number & ClickHouseInt<"int16">;
  NetMinor: number & ClickHouseInt<"int16">;
  UserAgentMajor: number & ClickHouseInt<"int16">;
  UserAgentMinor: string;
  CookieEnable: number & ClickHouseInt<"int16">;
  JavascriptEnable: number & ClickHouseInt<"int16">;
  IsMobile: number & ClickHouseInt<"int16">;
  MobilePhone: number & ClickHouseInt<"int16">;
  MobilePhoneModel: string;
  Params: string;
  IPNetworkID: number & ClickHouseInt<"int32">;
  TraficSourceID: number & ClickHouseInt<"int16">;
  SearchEngineID: number & ClickHouseInt<"int16">;
  SearchPhrase: string;
  AdvEngineID: number & ClickHouseInt<"int16">;
  IsArtifical: number & ClickHouseInt<"int16">;
  WindowClientWidth: number & ClickHouseInt<"int16">;
  WindowClientHeight: number & ClickHouseInt<"int16">;
  ClientTimeZone: number & ClickHouseInt<"int16">;
  ClientEventTime: Date;
  SilverlightVersion1: number & ClickHouseInt<"int16">;
  SilverlightVersion2: number & ClickHouseInt<"int16">;
  SilverlightVersion3: number & ClickHouseInt<"int32">;
  SilverlightVersion4: number & ClickHouseInt<"int16">;
  PageCharset: string;
  CodeVersion: number & ClickHouseInt<"int32">;
  IsLink: number & ClickHouseInt<"int16">;
  IsDownload: number & ClickHouseInt<"int16">;
  IsNotBounce: number & ClickHouseInt<"int16">;
  FUniqID: number & ClickHouseInt<"int64">;
  OriginalURL: string;
  HID: number & ClickHouseInt<"int32">;
  IsOldCounter: number & ClickHouseInt<"int16">;
  IsEvent: number & ClickHouseInt<"int16">;
  IsParameter: number & ClickHouseInt<"int16">;
  DontCountHits: number & ClickHouseInt<"int16">;
  WithHash: number & ClickHouseInt<"int16">;
  HitColor: string;
  LocalEventTime: Date;
  Age: number & ClickHouseInt<"int16">;
  Sex: number & ClickHouseInt<"int16">;
  Income: number & ClickHouseInt<"int16">;
  Interests: number & ClickHouseInt<"int16">;
  Robotness: number & ClickHouseInt<"int16">;
  RemoteIP: number & ClickHouseInt<"int32">;
  WindowName: number & ClickHouseInt<"int32">;
  OpenerName: number & ClickHouseInt<"int32">;
  HistoryLength: number & ClickHouseInt<"int16">;
  BrowserLanguage: string;
  BrowserCountry: string;
  SocialNetwork: string;
  SocialAction: string;
  HTTPError: number & ClickHouseInt<"int16">;
  SendTiming: number & ClickHouseInt<"int32">;
  DNSTiming: number & ClickHouseInt<"int32">;
  ConnectTiming: number & ClickHouseInt<"int32">;
  ResponseStartTiming: number & ClickHouseInt<"int32">;
  ResponseEndTiming: number & ClickHouseInt<"int32">;
  FetchTiming: number & ClickHouseInt<"int32">;
  SocialSourceNetworkID: number & ClickHouseInt<"int16">;
  SocialSourcePage: string;
  ParamPrice: number & ClickHouseInt<"int64">;
  ParamOrderID: string;
  ParamCurrency: string;
  ParamCurrencyID: number & ClickHouseInt<"int16">;
  OpenstatServiceName: string;
  OpenstatCampaignID: string;
  OpenstatAdID: string;
  OpenstatSourceID: string;
  UTMSource: string;
  UTMMedium: string;
  UTMCampaign: string;
  UTMContent: string;
  UTMTerm: string;
  FromTag: string;
  HasGCLID: number & ClickHouseInt<"int16">;
  RefererHash: number & ClickHouseInt<"int64">;
  URLHash: number & ClickHouseInt<"int64">;
  CLID: number & ClickHouseInt<"int32">;
}

export interface hn_styles {
  name: string;
  vec: (number & ClickHouseInt<"uint32">)[];
}

export interface lineorder {
  LO_ORDERKEY: number & ClickHouseInt<"uint32">;
  LO_LINENUMBER: number & ClickHouseInt<"uint8">;
  LO_CUSTKEY: number & ClickHouseInt<"uint32">;
  LO_PARTKEY: number & ClickHouseInt<"uint32">;
  LO_SUPPKEY: number & ClickHouseInt<"uint32">;
  LO_ORDERDATE: string & typia.tags.Format<"date"> & ClickHouseByteSize<2>;
  LO_ORDERPRIORITY: string;
  LO_SHIPPRIORITY: number & ClickHouseInt<"uint8">;
  LO_QUANTITY: number & ClickHouseInt<"uint8">;
  LO_EXTENDEDPRICE: number & ClickHouseInt<"uint32">;
  LO_ORDTOTALPRICE: number & ClickHouseInt<"uint32">;
  LO_DISCOUNT: number & ClickHouseInt<"uint8">;
  LO_REVENUE: number & ClickHouseInt<"uint32">;
  LO_SUPPLYCOST: number & ClickHouseInt<"uint32">;
  LO_TAX: number & ClickHouseInt<"uint8">;
  LO_COMMITDATE: string & typia.tags.Format<"date"> & ClickHouseByteSize<2>;
  LO_SHIPMODE: string;
}

export interface loc_stats {
  repo_name: string;
  language: string;
  path: string;
  file: string;
  lines: number & ClickHouseInt<"uint32">;
  code: number & ClickHouseInt<"uint32">;
  comments: number & ClickHouseInt<"uint32">;
  blanks: number & ClickHouseInt<"uint32">;
  complexity: number & ClickHouseInt<"uint32">;
  bytes: number & ClickHouseInt<"uint64">;
}

export interface menu {
  id: number & ClickHouseInt<"uint32">;
  name: string;
  sponsor: string;
  event: string;
  venue: string;
  place: string;
  physical_description: string;
  occasion: string;
  notes: string;
  call_number: string;
  keywords: string;
  language: string;
  date: string;
  location: string;
  location_type: string;
  currency: string;
  currency_symbol: string;
  status: string;
  page_count: number & ClickHouseInt<"uint16">;
  dish_count: number & ClickHouseInt<"uint16">;
}

export interface menu_item {
  id: number & ClickHouseInt<"uint32">;
  menu_page_id: number & ClickHouseInt<"uint32">;
  price: string & ClickHouseDecimal<18, 3>;
  high_price: string & ClickHouseDecimal<18, 3>;
  dish_id: number & ClickHouseInt<"uint32">;
  created_at: Date;
  updated_at: Date;
  xpos: number;
  ypos: number;
}

export interface menu_item_denorm {
  price: string & ClickHouseDecimal<18, 3>;
  high_price: string & ClickHouseDecimal<18, 3>;
  created_at: Date;
  updated_at: Date;
  xpos: number;
  ypos: number;
  dish_id: number & ClickHouseInt<"uint32">;
  dish_name: string;
  dish_description: string;
  dish_menus_appeared: number & ClickHouseInt<"uint32">;
  dish_times_appeared: number & ClickHouseInt<"int32">;
  dish_first_appeared: number & ClickHouseInt<"uint16">;
  dish_last_appeared: number & ClickHouseInt<"uint16">;
  dish_lowest_price: string & ClickHouseDecimal<18, 3>;
  dish_highest_price: string & ClickHouseDecimal<18, 3>;
  menu_id: number & ClickHouseInt<"uint32">;
  menu_name: string;
  menu_sponsor: string;
  menu_event: string;
  menu_venue: string;
  menu_place: string;
  menu_physical_description: string;
  menu_occasion: string;
  menu_notes: string;
  menu_call_number: string;
  menu_keywords: string;
  menu_language: string;
  menu_date: string;
  menu_location: string;
  menu_location_type: string;
  menu_currency: string;
  menu_currency_symbol: string;
  menu_status: string;
  menu_page_count: number & ClickHouseInt<"uint16">;
  menu_dish_count: number & ClickHouseInt<"uint16">;
}

export interface menu_page {
  id: number & ClickHouseInt<"uint32">;
  menu_id: number & ClickHouseInt<"uint32">;
  page_number: number & ClickHouseInt<"uint16">;
  image_id: string;
  full_height: number & ClickHouseInt<"uint16">;
  full_width: number & ClickHouseInt<"uint16">;
  uuid: string & typia.tags.Format<"uuid">;
}

export interface minicrawl {
  rank: number & ClickHouseInt<"uint32">;
  domain: string;
  log: string;
  content: string;
  size: number & ClickHouseInt<"uint64">;
  is_utf8: boolean;
  text: string;
}

export interface newswire {
  article: string;
  byline: string;
  dates: string[];
  newspaper_metadata: ({
    lccn: string;
    newspaper_city: string;
    newspaper_state: string;
    newspaper_title: string;
  } & ClickHouseNamedTuple)[];
  antitrust: number & ClickHouseInt<"int64">;
  civil_rights: number & ClickHouseInt<"int64">;
  crime: number & ClickHouseInt<"int64">;
  govt_regulation: number & ClickHouseInt<"int64">;
  labor_movement: number & ClickHouseInt<"int64">;
  politics: number & ClickHouseInt<"int64">;
  protests: number & ClickHouseInt<"int64">;
  ca_topic: string;
  ner_words: string[];
  ner_labels: string[];
  wire_city: string;
  wire_state: string;
  wire_country: string;
  lat: number;
  lon: number;
  wire_location_notes: string;
  people_mentioned: ({
    person_gender: string;
    person_name: string;
    person_occupation: string;
    wikidata_id: string;
  } & ClickHouseNamedTuple)[];
  cluster_size: number & ClickHouseInt<"int64">;
  year: number & ClickHouseInt<"int64">;
}

export interface ontime {
  Year: number & ClickHouseInt<"uint16">;
  Quarter: number & ClickHouseInt<"uint8">;
  Month: number & ClickHouseInt<"uint8">;
  DayofMonth: number & ClickHouseInt<"uint8">;
  DayOfWeek: number & ClickHouseInt<"uint8">;
  FlightDate: string & typia.tags.Format<"date"> & ClickHouseByteSize<2>;
  Reporting_Airline: string;
  DOT_ID_Reporting_Airline: number & ClickHouseInt<"int32">;
  IATA_CODE_Reporting_Airline: string;
  Tail_Number: string;
  Flight_Number_Reporting_Airline: string;
  OriginAirportID: number & ClickHouseInt<"int32">;
  OriginAirportSeqID: number & ClickHouseInt<"int32">;
  OriginCityMarketID: number & ClickHouseInt<"int32">;
  Origin: string;
  OriginCityName: string;
  OriginState: string;
  OriginStateFips: string;
  OriginStateName: string;
  OriginWac: number & ClickHouseInt<"int32">;
  DestAirportID: number & ClickHouseInt<"int32">;
  DestAirportSeqID: number & ClickHouseInt<"int32">;
  DestCityMarketID: number & ClickHouseInt<"int32">;
  Dest: string;
  DestCityName: string;
  DestState: string;
  DestStateFips: string;
  DestStateName: string;
  DestWac: number & ClickHouseInt<"int32">;
  CRSDepTime: number & ClickHouseInt<"int32">;
  DepTime: number & ClickHouseInt<"int32">;
  DepDelay: number & ClickHouseInt<"int32">;
  DepDelayMinutes: number & ClickHouseInt<"int32">;
  DepDel15: number & ClickHouseInt<"int32">;
  DepartureDelayGroups: string;
  DepTimeBlk: string;
  TaxiOut: number & ClickHouseInt<"int32">;
  WheelsOff: string;
  WheelsOn: string;
  TaxiIn: number & ClickHouseInt<"int32">;
  CRSArrTime: number & ClickHouseInt<"int32">;
  ArrTime: number & ClickHouseInt<"int32">;
  ArrDelay: number & ClickHouseInt<"int32">;
  ArrDelayMinutes: number & ClickHouseInt<"int32">;
  ArrDel15: number & ClickHouseInt<"int32">;
  ArrivalDelayGroups: string;
  ArrTimeBlk: string;
  Cancelled: number & ClickHouseInt<"int8">;
  CancellationCode: string;
  Diverted: number & ClickHouseInt<"int8">;
  CRSElapsedTime: number & ClickHouseInt<"int32">;
  ActualElapsedTime: number & ClickHouseInt<"int32">;
  AirTime: number & ClickHouseInt<"int32">;
  Flights: number & ClickHouseInt<"int32">;
  Distance: number & ClickHouseInt<"int32">;
  DistanceGroup: number & ClickHouseInt<"int8">;
  CarrierDelay: number & ClickHouseInt<"int32">;
  WeatherDelay: number & ClickHouseInt<"int32">;
  NASDelay: number & ClickHouseInt<"int32">;
  SecurityDelay: number & ClickHouseInt<"int32">;
  LateAircraftDelay: number & ClickHouseInt<"int32">;
  FirstDepTime: number & ClickHouseInt<"int16">;
  TotalAddGTime: number & ClickHouseInt<"int16">;
  LongestAddGTime: number & ClickHouseInt<"int16">;
  DivAirportLandings: number & ClickHouseInt<"int8">;
  DivReachedDest: number & ClickHouseInt<"int8">;
  DivActualElapsedTime: number & ClickHouseInt<"int16">;
  DivArrDelay: number & ClickHouseInt<"int16">;
  DivDistance: number & ClickHouseInt<"int16">;
  Div1Airport: string;
  Div1AirportID: number & ClickHouseInt<"int32">;
  Div1AirportSeqID: number & ClickHouseInt<"int32">;
  Div1WheelsOn: number & ClickHouseInt<"int16">;
  Div1TotalGTime: number & ClickHouseInt<"int16">;
  Div1LongestGTime: number & ClickHouseInt<"int16">;
  Div1WheelsOff: number & ClickHouseInt<"int16">;
  Div1TailNum: string;
  Div2Airport: string;
  Div2AirportID: number & ClickHouseInt<"int32">;
  Div2AirportSeqID: number & ClickHouseInt<"int32">;
  Div2WheelsOn: number & ClickHouseInt<"int16">;
  Div2TotalGTime: number & ClickHouseInt<"int16">;
  Div2LongestGTime: number & ClickHouseInt<"int16">;
  Div2WheelsOff: number & ClickHouseInt<"int16">;
  Div2TailNum: string;
  Div3Airport: string;
  Div3AirportID: number & ClickHouseInt<"int32">;
  Div3AirportSeqID: number & ClickHouseInt<"int32">;
  Div3WheelsOn: number & ClickHouseInt<"int16">;
  Div3TotalGTime: number & ClickHouseInt<"int16">;
  Div3LongestGTime: number & ClickHouseInt<"int16">;
  Div3WheelsOff: number & ClickHouseInt<"int16">;
  Div3TailNum: string;
  Div4Airport: string;
  Div4AirportID: number & ClickHouseInt<"int32">;
  Div4AirportSeqID: number & ClickHouseInt<"int32">;
  Div4WheelsOn: number & ClickHouseInt<"int16">;
  Div4TotalGTime: number & ClickHouseInt<"int16">;
  Div4LongestGTime: number & ClickHouseInt<"int16">;
  Div4WheelsOff: number & ClickHouseInt<"int16">;
  Div4TailNum: string;
  Div5Airport: string;
  Div5AirportID: number & ClickHouseInt<"int32">;
  Div5AirportSeqID: number & ClickHouseInt<"int32">;
  Div5WheelsOn: number & ClickHouseInt<"int16">;
  Div5TotalGTime: number & ClickHouseInt<"int16">;
  Div5LongestGTime: number & ClickHouseInt<"int16">;
  Div5WheelsOff: number & ClickHouseInt<"int16">;
  Div5TailNum: string;
}

export interface opensky {
  callsign: string;
  number: string;
  icao24: string;
  registration: string;
  typecode: string;
  origin: string;
  destination: string;
  firstseen: Date;
  lastseen: Date;
  day: Date;
  latitude_1: number;
  longitude_1: number;
  altitude_1: number;
  latitude_2: number;
  longitude_2: number;
  altitude_2: number;
}

export interface pypi {
  project_name: string;
  project_version: string;
  project_release: string;
  uploaded_on: string & typia.tags.Format<"date-time"> & ClickHousePrecision<3>;
  path: string;
  archive_path: string;
  size: number & ClickHouseInt<"uint64">;
  hash: string;
  skip_reason: string;
  lines: number & ClickHouseInt<"uint64">;
  repository: number & ClickHouseInt<"uint32">;
}

export interface query_metrics_v2 {
  event_date: string & typia.tags.Format<"date"> & ClickHouseByteSize<2>;
  event_time: Date;
  pr_number: number & ClickHouseInt<"uint32">;
  old_sha: string;
  new_sha: string;
  test: string;
  query_index: number & ClickHouseInt<"uint32">;
  query_display_name: string;
  metric: string;
  old_value: number;
  new_value: number;
  diff: number;
  stat_threshold: number;
}

export interface rdns {
  timestamp: Date;
  address: string & typia.tags.Format<"ipv4">;
  domain: string;
}

export interface recipes {
  title: string;
  ingredients: string[];
  directions: string[];
  link: string;
  source: string;
  NER: string[];
}

export interface repos {
  full_name: string;
  owner_type: string;
  description: string;
  fork: boolean;
  created_at: Date;
  updated_at: Date;
  pushed_at: Date;
  homepage: string;
  size: number & ClickHouseInt<"uint64">;
  stargazers_count: number & ClickHouseInt<"uint32">;
  forks_count: number & ClickHouseInt<"uint32">;
  subscribers_count: number & ClickHouseInt<"uint32">;
  language: string;
  has_issues: boolean;
  has_projects: boolean;
  has_downloads: boolean;
  has_wiki: boolean;
  has_pages: boolean;
  archived: boolean;
  disabled: boolean;
  is_template: boolean;
  allow_forking: boolean;
  open_issues_count: number & ClickHouseInt<"uint32">;
  license_key: string;
  topics: string[];
  default_branch: string;
}

export interface repos_history {
  time: Date;
  id: number & ClickHouseInt<"uint64">;
  name: string;
  owner: {
    login: string;
    id: number & ClickHouseInt<"uint64">;
    type: string;
    site_admin: boolean;
  } & ClickHouseNamedTuple;
  full_name: string;
  description: string;
  fork: boolean;
  created_at: Date;
  updated_at: Date;
  pushed_at: Date;
  homepage: string;
  size: number & ClickHouseInt<"uint64">;
  stargazers_count: number & ClickHouseInt<"uint32">;
  language: string;
  has_issues: boolean;
  has_projects: boolean;
  has_downloads: boolean;
  has_wiki: boolean;
  has_pages: boolean;
  has_discussions: boolean;
  forks_count: number & ClickHouseInt<"uint32">;
  mirror_url: string;
  archived: boolean;
  disabled: boolean;
  open_issues_count: number & ClickHouseInt<"uint32">;
  license: {
    key: string;
    name: string;
    spdx_id: string;
  } & ClickHouseNamedTuple;
  allow_forking: boolean;
  is_template: boolean;
  web_commit_signoff_required: boolean;
  topics: string[];
  visibility: string;
  default_branch: string;
  network_count: number & ClickHouseInt<"uint32">;
  subscribers_count: number & ClickHouseInt<"uint32">;
}

export interface repos_raw {
  data: string;
}

export interface run_attributes_v1 {
  old_sha: string;
  new_sha: string;
  metric: string;
  metric_value: string;
}

export interface stock {
  symbol: string;
  date: string & typia.tags.Format<"date"> & ClickHouseByteSize<2>;
  price: string & ClickHouseDecimal<9, 3>;
  volume: number & ClickHouseInt<"uint32">;
  open: string & ClickHouseDecimal<9, 3>;
  low: string & ClickHouseDecimal<9, 3>;
  high: string & ClickHouseDecimal<9, 3>;
}

export interface tranco {
  date: string & typia.tags.Format<"date"> & ClickHouseByteSize<2>;
  rank: number & ClickHouseInt<"uint32">;
  domain: string;
}

export interface trips {
  trip_id: number & ClickHouseInt<"uint32">;
  vendor_id: VendorId;
  pickup_date: string & typia.tags.Format<"date"> & ClickHouseByteSize<2>;
  pickup_datetime: Date;
  dropoff_date: string & typia.tags.Format<"date"> & ClickHouseByteSize<2>;
  dropoff_datetime: Date;
  store_and_fwd_flag: number & ClickHouseInt<"uint8">;
  rate_code_id: number & ClickHouseInt<"uint8">;
  pickup_longitude: number;
  pickup_latitude: number;
  dropoff_longitude: number;
  dropoff_latitude: number;
  passenger_count: number & ClickHouseInt<"uint8">;
  trip_distance: number;
  fare_amount: number & typia.tags.Type<"float">;
  extra: number & typia.tags.Type<"float">;
  mta_tax: number & typia.tags.Type<"float">;
  tip_amount: number & typia.tags.Type<"float">;
  tolls_amount: number & typia.tags.Type<"float">;
  ehail_fee: number & typia.tags.Type<"float">;
  improvement_surcharge: number & typia.tags.Type<"float">;
  total_amount: number & typia.tags.Type<"float">;
  payment_type_: PaymentType;
  trip_type: number & ClickHouseInt<"uint8">;
  pickup: string;
  dropoff: string;
  cab_type: CabType;
  pickup_nyct2010_gid: number & ClickHouseInt<"int8">;
  pickup_ctlabel: number & typia.tags.Type<"float">;
  pickup_borocode: number & ClickHouseInt<"int8">;
  pickup_ct2010: string;
  pickup_boroct2010: string;
  pickup_cdeligibil: string;
  pickup_ntacode: string;
  pickup_ntaname: string;
  pickup_puma: number & ClickHouseInt<"uint16">;
  dropoff_nyct2010_gid: number & ClickHouseInt<"uint8">;
  dropoff_ctlabel: number & typia.tags.Type<"float">;
  dropoff_borocode: number & ClickHouseInt<"uint8">;
  dropoff_ct2010: string;
  dropoff_boroct2010: string;
  dropoff_cdeligibil: string;
  dropoff_ntacode: string;
  dropoff_ntaname: string;
  dropoff_puma: number & ClickHouseInt<"uint16">;
}

export interface uk_price_paid {
  price: number & ClickHouseInt<"uint32">;
  date: string & typia.tags.Format<"date"> & ClickHouseByteSize<2>;
  postcode1: string;
  postcode2: string;
  type: Type2;
  is_new: number & ClickHouseInt<"uint8">;
  duration: Duration;
  addr1: string;
  addr2: string;
  street: string;
  locality: string;
  town: string;
  district: string;
  county: string;
}

export interface version_history {
  check_start_time: Date;
  pull_request_number: number & ClickHouseInt<"uint32">;
  pull_request_url: string;
  commit_sha: string;
  parent_commits_sha: string[];
  commit_url: string;
  version: string;
  docker_tag: string;
  git_ref: string;
}

export interface wikistat {
  time: Date;
  project: string;
  subproject: string;
  path: string;
  hits: number & ClickHouseInt<"uint64">;
}

export interface workflow_jobs {
  id: number & ClickHouseInt<"uint64">;
  run_id: number & ClickHouseInt<"uint64">;
  workflow_name: string;
  head_branch: string;
  run_url: string;
  run_attempt: number & ClickHouseInt<"uint16">;
  node_id: string;
  head_sha: string;
  url: string;
  html_url: string;
  status: Status;
  conclusion: string;
  started_at: Date;
  completed_at: Date;
  name: string;
  steps: number & ClickHouseInt<"uint16">;
  check_run_url: string;
  labels: string[];
  runner_id: number & ClickHouseInt<"uint64">;
  runner_name: string;
  runner_group_id: number & ClickHouseInt<"uint64">;
  runner_group_name: string;
  repository: string;
  updated_at: Date;
}

export const ActorsPipeline = new IngestPipeline<actors>("actors", {
  table: true,
  stream: true,
  ingest: true,
});

export const BenchmarkResultsPipeline = new IngestPipeline<benchmark_results>(
  "benchmark_results",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);

export const BenchmarkRunsPipeline = new IngestPipeline<benchmark_runs>(
  "benchmark_runs",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);

export const CellTowersPipeline = new IngestPipeline<cell_towers>(
  "cell_towers",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);

export const ChecksPipeline = new IngestPipeline<checks>("checks", {
  table: true,
  stream: true,
  ingest: true,
});

export const CiscoUmbrellaPipeline = new IngestPipeline<cisco_umbrella>(
  "cisco_umbrella",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);

export const CovidPipeline = new IngestPipeline<covid>("covid", {
  table: true,
  stream: true,
  ingest: true,
});

export const DishPipeline = new IngestPipeline<dish>("dish", {
  table: true,
  stream: true,
  ingest: true,
});

export const DnsPipeline = new IngestPipeline<dns>("dns", {
  table: true,
  stream: true,
  ingest: true,
});

export const Dns2Pipeline = new IngestPipeline<dns2>("dns2", {
  table: true,
  stream: true,
  ingest: true,
});

export const GithubEventsPipeline = new IngestPipeline<github_events>(
  "github_events",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);

export const HackernewsPipeline = new IngestPipeline<hackernews>("hackernews", {
  table: true,
  stream: true,
  ingest: true,
});

export const HackernewsChangesItemsPipeline =
  new IngestPipeline<hackernews_changes_items>("hackernews_changes_items", {
    table: true,
    stream: true,
    ingest: true,
  });

export const HackernewsChangesProfilesPipeline =
  new IngestPipeline<hackernews_changes_profiles>(
    "hackernews_changes_profiles",
    {
      table: true,
      stream: true,
      ingest: true,
    },
  );

export const HackernewsHistoryPipeline = new IngestPipeline<hackernews_history>(
  "hackernews_history",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);

export const HackernewsTopPipeline = new IngestPipeline<hackernews_top>(
  "hackernews_top",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);

export const HitsPipeline = new IngestPipeline<hits>("hits", {
  table: true,
  stream: true,
  ingest: true,
});

export const Hits2Pipeline = new IngestPipeline<hits2>("hits2", {
  table: true,
  stream: true,
  ingest: true,
});

export const HnStylesPipeline = new IngestPipeline<hn_styles>("hn_styles", {
  table: true,
  stream: true,
  ingest: true,
});

export const LineorderPipeline = new IngestPipeline<lineorder>("lineorder", {
  table: true,
  stream: true,
  ingest: true,
});

export const LocStatsPipeline = new IngestPipeline<loc_stats>("loc_stats", {
  table: true,
  stream: true,
  ingest: true,
});

export const MenuPipeline = new IngestPipeline<menu>("menu", {
  table: true,
  stream: true,
  ingest: true,
});

export const MenuItemPipeline = new IngestPipeline<menu_item>("menu_item", {
  table: true,
  stream: true,
  ingest: true,
});

export const MenuItemDenormPipeline = new IngestPipeline<menu_item_denorm>(
  "menu_item_denorm",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);

export const MenuPagePipeline = new IngestPipeline<menu_page>("menu_page", {
  table: true,
  stream: true,
  ingest: true,
});

export const MinicrawlPipeline = new IngestPipeline<minicrawl>("minicrawl", {
  table: true,
  stream: true,
  ingest: true,
});

export const NewswirePipeline = new IngestPipeline<newswire>("newswire", {
  table: true,
  stream: true,
  ingest: true,
});

export const OntimePipeline = new IngestPipeline<ontime>("ontime", {
  table: true,
  stream: true,
  ingest: true,
});

export const OpenskyPipeline = new IngestPipeline<opensky>("opensky", {
  table: true,
  stream: true,
  ingest: true,
});

export const PypiPipeline = new IngestPipeline<pypi>("pypi", {
  table: true,
  stream: true,
  ingest: true,
});

export const QueryMetricsV2Pipeline = new IngestPipeline<query_metrics_v2>(
  "query_metrics_v2",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);

export const RdnsPipeline = new IngestPipeline<rdns>("rdns", {
  table: true,
  stream: true,
  ingest: true,
});

export const RecipesPipeline = new IngestPipeline<recipes>("recipes", {
  table: true,
  stream: true,
  ingest: true,
});

export const ReposPipeline = new IngestPipeline<repos>("repos", {
  table: true,
  stream: true,
  ingest: true,
});

export const ReposHistoryPipeline = new IngestPipeline<repos_history>(
  "repos_history",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);

export const ReposRawPipeline = new IngestPipeline<repos_raw>("repos_raw", {
  table: true,
  stream: true,
  ingest: true,
});

export const RunAttributesV1Pipeline = new IngestPipeline<run_attributes_v1>(
  "run_attributes_v1",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);

export const StockPipeline = new IngestPipeline<stock>("stock", {
  table: true,
  stream: true,
  ingest: true,
});

export const TrancoPipeline = new IngestPipeline<tranco>("tranco", {
  table: true,
  stream: true,
  ingest: true,
});

export const TripsPipeline = new IngestPipeline<trips>("trips", {
  table: true,
  stream: true,
  ingest: true,
});

export const UkPricePaidPipeline = new IngestPipeline<uk_price_paid>(
  "uk_price_paid",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);

export const VersionHistoryPipeline = new IngestPipeline<version_history>(
  "version_history",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);

export const WikistatPipeline = new IngestPipeline<wikistat>("wikistat", {
  table: true,
  stream: true,
  ingest: true,
});

export const WorkflowJobsPipeline = new IngestPipeline<workflow_jobs>(
  "workflow_jobs",
  {
    table: true,
    stream: true,
    ingest: true,
  },
);
