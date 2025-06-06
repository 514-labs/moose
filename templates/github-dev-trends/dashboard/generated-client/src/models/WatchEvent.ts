/* tslint:disable */
/* eslint-disable */
/**
 * live-dev-trends API
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0
 *
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { mapValues } from "../runtime";
/**
 *
 * @export
 * @interface WatchEvent
 */
export interface WatchEvent {
  /**
   *
   * @type {string}
   * @memberof WatchEvent
   */
  actorLogin: string;
  /**
   *
   * @type {number}
   * @memberof WatchEvent
   */
  repoId: number;
  /**
   *
   * @type {string}
   * @memberof WatchEvent
   */
  eventId: string;
  /**
   *
   * @type {string}
   * @memberof WatchEvent
   */
  repoUrl: string;
  /**
   *
   * @type {string}
   * @memberof WatchEvent
   */
  repoName: string;
  /**
   *
   * @type {string}
   * @memberof WatchEvent
   */
  createdAt: string;
  /**
   *
   * @type {number}
   * @memberof WatchEvent
   */
  actorId: number;
  /**
   *
   * @type {string}
   * @memberof WatchEvent
   */
  actorAvatarUrl: string;
  /**
   *
   * @type {string}
   * @memberof WatchEvent
   */
  actorUrl: string;
}

/**
 * Check if a given object implements the WatchEvent interface.
 */
export function instanceOfWatchEvent(value: object): value is WatchEvent {
  if (!("actorLogin" in value) || value["actorLogin"] === undefined)
    return false;
  if (!("repoId" in value) || value["repoId"] === undefined) return false;
  if (!("eventId" in value) || value["eventId"] === undefined) return false;
  if (!("repoUrl" in value) || value["repoUrl"] === undefined) return false;
  if (!("repoName" in value) || value["repoName"] === undefined) return false;
  if (!("createdAt" in value) || value["createdAt"] === undefined) return false;
  if (!("actorId" in value) || value["actorId"] === undefined) return false;
  if (!("actorAvatarUrl" in value) || value["actorAvatarUrl"] === undefined)
    return false;
  if (!("actorUrl" in value) || value["actorUrl"] === undefined) return false;
  return true;
}

export function WatchEventFromJSON(json: any): WatchEvent {
  return WatchEventFromJSONTyped(json, false);
}

export function WatchEventFromJSONTyped(
  json: any,
  ignoreDiscriminator: boolean,
): WatchEvent {
  if (json == null) {
    return json;
  }
  return {
    actorLogin: json["actorLogin"],
    repoId: json["repoId"],
    eventId: json["eventId"],
    repoUrl: json["repoUrl"],
    repoName: json["repoName"],
    createdAt: json["createdAt"],
    actorId: json["actorId"],
    actorAvatarUrl: json["actorAvatarUrl"],
    actorUrl: json["actorUrl"],
  };
}

export function WatchEventToJSON(json: any): WatchEvent {
  return WatchEventToJSONTyped(json, false);
}

export function WatchEventToJSONTyped(
  value?: WatchEvent | null,
  ignoreDiscriminator: boolean = false,
): any {
  if (value == null) {
    return value;
  }

  return {
    actorLogin: value["actorLogin"],
    repoId: value["repoId"],
    eventId: value["eventId"],
    repoUrl: value["repoUrl"],
    repoName: value["repoName"],
    createdAt: value["createdAt"],
    actorId: value["actorId"],
    actorAvatarUrl: value["actorAvatarUrl"],
    actorUrl: value["actorUrl"],
  };
}
