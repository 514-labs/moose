import { FieldValues, UseFormReturn, useFieldArray } from "react-hook-form";
import { FormField } from "../ui/form";

import MultiSelectForm from "./multi-select-form";
import { useQuery } from "@tanstack/react-query";
import { getMetricAttributes } from "@/data-api";
import { createMultiSelectOptions } from "@/lib/utils";
import { Button } from "../ui/button";
import { XIcon } from "lucide-react";

export type MultiSelectList = { label: string; val: string }[];

interface GroupingFormProps {
  form: UseFormReturn<FieldValues, any, undefined>;
  index: number;
  metricIndex: number;
  metricName: string;
}

export default function GroupingForm({
  form,
  index,
  metricIndex,
  metricName,
}: FilterFormProps) {
  const { fields, remove } = useFieldArray({
    name: `list.${metricIndex}.grouping`,
    control: form.control,
  });
  const { isLoading, isError, data, error } = useQuery({
    queryKey: ["metricName", metricName],
    queryFn: () => getMetricAttributes(metricName),
    initialData: {},
  });

  if (isLoading) return <div>Loading...</div>;
  if (isError) return <div>Error: {error.message}</div>;

  const properties = createMultiSelectOptions(Object.keys(data));

  return (
    <div className="p-2">
      {fields.map((field, index) => {
        const operatorValues =
          data[form.getValues().list[metricIndex].grouping[index].property];

        const operatorOptions = operatorValues
          ? createMultiSelectOptions(operatorValues)
          : [];

        return (
          <div
            className="grid grid-cols-4 gap-4"
            key={`list.${metricIndex}.grouping.${index}`}
          >
            <FormField
              control={form.control}
              name={`list.${metricIndex}.grouping.${index}.property`}
              render={({ field }) => (
                <MultiSelectForm
                  index={index}
                  field={field}
                  placeholder="Select Metric"
                  remove={remove}
                  options={properties}
                />
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
        );
      })}
    </div>
  );
}
