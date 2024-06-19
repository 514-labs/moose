"use client";

import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Combobox } from "../createable-select";
import { getMetricCommonProperties } from "@/data-api";
import { FormField } from "../ui/form";
import { FieldValues, UseFormReturn } from "react-hook-form";
import { createMultiSelectOptions } from "@/lib/utils";

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
    enabled: isOpen,
    initialData: [],
  });

  console.log("optionsData", optionsData);

  const commonOptions = optionsData.map((option) => ({
    label: `(${option.count}) ${option.value}`,
    val: option.value != null ? option.value : "",
  }));

  console.log("commonOptions", commonOptions);
  return (
    <FormField
      control={form.control}
      name={`list.${metricIndex}.filter.${index}.value`}
      render={({ field }) => {
        return (
          <div className="w-24">
            <Combobox
              mode="multiple"
              options={commonOptions}
              onOpenChange={(value) => setIsOpen(value)}
              placeholder="Select option"
              selected={field.value ?? ""} // string or array
              onSelect={(value) => {
                console.log(value);
                return field.onChange(value);
              }}
              onCreate={(value) => {
                field.onChange(value);
              }}
            />
          </div>
        );
      }}
    />
  );
}
