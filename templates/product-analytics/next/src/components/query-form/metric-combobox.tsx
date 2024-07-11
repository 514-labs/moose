"use client";

import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Combobox } from "../createable-select";
import { getMetricCommonProperties } from "@/data-api";
import { FormField } from "../ui/form";
import { FieldValues, UseFormReturn } from "react-hook-form";

interface MetricComboBoxProps {
  form: UseFormReturn<FieldValues, any, undefined>;
  index: number;
  metricIndex: number;
}

export default function MetricComboBox({
  form,
  index,
  metricIndex,
}: MetricComboBoxProps) {
  const [isOpen, setIsOpen] = useState(false);

  const metricName = form.getValues().list[metricIndex].metricName;
  const property = form.getValues().list[metricIndex].filter[index].property;

  const filters = form
    .getValues()
    .list[metricIndex].filter.filter((_f, i) => index !== i);

  const filterEncoded = encodeURIComponent(JSON.stringify(filters));

  const {
    isLoading: optionsLoading,
    isError: isOptionsError,
    data: optionsData,
    error: optionsError,
  } = useQuery({
    queryKey: ["metric_properties", metricName, property, filterEncoded],
    queryFn: () =>
      getMetricCommonProperties({
        metricName,
        propertyName: property,
        filter: filterEncoded,
      }),
    enabled: isOpen && property != null,
    initialData: [],
  });

  if (optionsLoading) return <div>Loading...</div>;
  if (isOptionsError) return <div>Error</div>;

  const commonOptions = optionsData.map((option) => ({
    label: `(${option.count}) ${option.value}`,
    val: option.value != null ? option.value : "",
  }));

  return (
    <FormField
      control={form.control}
      name={`list.${metricIndex}.filter.${index}.value`}
      render={({ field }) => {
        return (
          <div className="w-24">
            <Combobox
              options={commonOptions}
              onOpenChange={(value) => setIsOpen(value)}
              placeholder="Select option"
              selected={field.value ?? ""} // string or array
              onSelect={field.onChange}
              onCreate={field.onChange}
            />
          </div>
        );
      }}
    />
  );
}
