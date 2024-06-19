"use client";

import {
  FieldValues,
  UseFieldArrayRemove,
  UseFormReturn,
} from "react-hook-form";
import { FormField } from "../ui/form";
import FilterForm from "./filter-form";
import MultiSelectForm from "./multi-select-form";
import { Button } from "../ui/button";
import { Plus, XIcon } from "lucide-react";
import GroupingForm from "./grouping-form";

interface MetricSelectProps {
  form: UseFormReturn<FieldValues, any, undefined>;
  index: number;
  options: { label: string; val: string }[];
  remove: UseFieldArrayRemove;
}
export default function MetricSelectForm({
  form,
  index,
  options,
  remove,
}: MetricSelectProps) {
  return (
    <div className="border-2 rounded-xl w-full p-2">
      <div className="w-full flex justify-between">
        <FormField
          control={form.control}
          name={`list.${index}.metricName`}
          render={({ field }) => (
            <div className="flex items-center">
              <span className="mr-4">Metric:</span>
              <MultiSelectForm
                index={index}
                field={field}
                placeholder="Select Metric"
                remove={remove}
                options={options}
                onChange={(_) => {
                  form.setValue(`list.${index}.filter`, []);
                  form.setValue(`list.${index}.grouping`, []);
                }}
              />
            </div>
          )}
        />
        <Button
          className="ml-4 p-2 h-fit"
          size={"sm"}
          variant={"ghost"}
          onClick={() => remove(index)}
        >
          <XIcon className="h-6 w-6" />
        </Button>
      </div>
      <Button
        type="button"
        variant="ghost"
        size="default"
        className="mt-2 rounded-xl  w-full justify-between"
        onClick={() => {
          form.setValue(`list.${index}.filter`, [
            ...form.getValues().list[index].filter,
            { property: null, value: null, operator: null },
          ]);
        }}
      >
        Add Filter
        <Plus />
      </Button>
      <FilterForm
        metricName={form.getValues().list[index].metricName}
        form={form}
        index={index}
        metricIndex={index}
      />
      <Button
        type="button"
        variant="ghost"
        size="default"
        className="mt-2 rounded-xl  w-full justify-between"
        onClick={() => {
          form.setValue(`list.${index}.grouping`, [
            ...form.getValues().list[index].grouping,
            { property: null },
          ]);
        }}
      >
        Add Grouping
        <Plus />
      </Button>
      <GroupingForm
        metricName={form.getValues().list[index].metricName}
        form={form}
        index={index}
        metricIndex={index}
      />
    </div>
  );
}
