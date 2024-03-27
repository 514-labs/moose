import {
  ControllerRenderProps,
  FieldValues,
  UseFormReturn,
} from "react-hook-form";
import { FormControl, FormField, FormItem, FormMessage } from "./ui/form";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";
import { MetricOptions } from "@/lib/form-types";
import { useState } from "react";

interface OptionsProps {
  form: UseFormReturn<FieldValues, any, undefined>;
  field: ControllerRenderProps<FieldValues, `list.${number}.metric`>;
  index: number;
}

const metricOptions = [
  { val: MetricOptions.Total_Events, label: "Total Events" },
  { val: MetricOptions.Total_Sessions, label: "Total Sessions" },
  { val: MetricOptions.Aggregated_Property, label: "AggregatedProperty" },
];

const aggregatedProperty = [
  { val: "sum", label: "sum" },
  { val: "min", label: "min" },
];

export default function MetricOptionsForm({
  field,
  form,
  index,
}: OptionsProps) {
  const [open, setOpen] = useState(false);
  return (
    <Select
      open={open}
      onValueChange={(val) => field.onChange(val)}
      defaultValue={field.value}
      value={field.value}
    >
      <FormControl>
        <SelectTrigger
          onClick={() => setOpen(true)}
          className="rounded-xl"
          value={field.value}
        >
          <SelectValue placeholder="Select Event" />
        </SelectTrigger>
      </FormControl>
      <SelectContent>
        {metricOptions.map((l, i) => (
          <OptionsSelect
            currVal={field.value}
            index={index}
            form={form}
            key={i}
            val={l.val}
            label={l.label}
            setOpen={setOpen}
          />
        ))}
      </SelectContent>
    </Select>
  );
}

interface AggregatedPropertyFormProps {
  form: UseFormReturn<FieldValues, any, undefined>;
  index: number;
  setOpen: (val: boolean) => void;
}
function AggregatedPropertyForm({
  index,
  form,
  setOpen,
}: AggregatedPropertyFormProps) {
  return (
    <FormField
      control={form.control}
      key={`${index}_metric.aggregatedProperty`}
      name={`list.${index}.metric.aggregatedProperty`}
      render={({ field }) => (
        <FormItem className="my-2">
          <div className="flex items-center">
            <Select
              onValueChange={(val) => field.onChange(val)}
              defaultValue={field.value}
              value={field.value}
            >
              <FormControl>
                <SelectTrigger className="rounded-xl" value={field.value}>
                  <SelectValue placeholder="Select Event" />
                </SelectTrigger>
              </FormControl>
              <SelectContent>
                {aggregatedProperty.map((l, i) => (
                  <SelectItem onClick={() => setOpen(false)} value={l.val}>
                    {l.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <FormMessage />
        </FormItem>
      )}
    />
  );
}

function OptionsSelect({
  form,
  val,
  label,
  index,
  setOpen,
  currVal,
}: {
  form: UseFormReturn<FieldValues, any, undefined>;
  val: string;
  label: string;
  index: number;
  currVal: string;
  setOpen: (val: boolean) => void;
}) {
  switch (val) {
    case MetricOptions.Total_Events:
      return (
        <SelectItem onClick={() => setOpen(false)} value={val}>
          {label}
        </SelectItem>
      );
    case MetricOptions.Total_Sessions:
      return (
        <SelectItem onClick={() => setOpen(false)} value={val}>
          {label}
        </SelectItem>
      );
    case MetricOptions.Aggregated_Property:
      return (
        <div>
          <SelectItem onClick={() => setOpen(true)} value={val}>
            {label}
          </SelectItem>
          {currVal == MetricOptions.Aggregated_Property && (
            <AggregatedPropertyForm
              setOpen={setOpen}
              form={form}
              index={index}
            />
          )}
        </div>
      );
    default:
      return <SelectItem value={val}>{label}</SelectItem>;
  }
}
