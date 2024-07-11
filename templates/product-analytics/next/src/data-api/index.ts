import { MetricList } from "@/app/types";
import { QueryFormData } from "@/components/query-form/query-form";
import { DateRange, getRangeDate } from "@/insights/time-query";

export async function getMetricList() {
  const response = await fetch("http://localhost:4000/consumption/metric_list");

  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  const data = (await response.json()) as MetricList;

  return data.map(({ metricName }) => ({
    val: metricName,
    label: metricName,
  }));
}

export interface MetricAttributes {
  [key: string]: string[];
}

export async function getMetricAttributes(metricName: string) {
  const response = await fetch(
    "http://localhost:4000/consumption/metric_attributes?metricName=" +
      metricName,
  );

  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  return (await response.json()) as MetricAttributes;
}

export interface MetricQuery {
  query: QueryFormData;
  range: DateRange;
  orderBy: { id: string; desc: boolean }[];
}

export async function getMetric({ query, range, orderBy }: MetricQuery) {
  const encodedQuery = encodeURIComponent(JSON.stringify(query));
  const url = new URL("http://localhost:4000/consumption/metric_query");
  const params = new URLSearchParams();
  if (query) {
    params.append("query", encodedQuery);
  }
  if (range) {
    const { from, to } = getRangeDate(range);
    params.append("from", from.toString());
    params.append("to", to.toString());
  }
  if (orderBy?.length > 0) {
    params.append("orderBy", orderBy[0].id);
    params.append("desc", orderBy[0].desc.toString());
  }
  url.search = params.toString();
  const response = await fetch(url.toString());

  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  return await response.json();
}

export interface QueryParams {
  metricName: string;
  propertyName: string;
  filter: string;
}

export async function getMetricCommonProperties({
  metricName,
  propertyName,
  filter,
}: QueryParams) {
  const response = await fetch(
    `http://localhost:4000/consumption/metric_common_properties?metricName=${metricName}&propertyName=${propertyName}&filters=${filter}`,
  );

  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  return await response.json();
}

export interface MetricQuery {
  query: QueryFormData;
  range: DateRange;
}

export async function getMetricTimeSeries({ query, range }: MetricQuery) {
  const encodedQuery = encodeURIComponent(JSON.stringify(query));

  const url = new URL("http://localhost:4000/consumption/metric_timeseries");
  const params = new URLSearchParams();
  if (query) {
    params.append("query", encodedQuery);
  }
  if (range) {
    const { from, to } = getRangeDate(range);
    params.append("from", from.toString());
    params.append("to", to.toString());
  }
  url.search = params.toString();
  const response = await fetch(url.toString());

  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  const data = await response.json();

  return data.map((d: any) => ({
    ...d,
    timeseries: d.timeseries.map((point) => [
      new Date(point[0]),
      parseInt(point[1]),
    ]),
  }));
}
