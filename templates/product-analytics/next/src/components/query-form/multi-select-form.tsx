"use client";

import { XIcon } from "lucide-react";
import { Button } from "../ui/button";
import { FormControl, FormItem, FormMessage } from "../ui/form";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import {
  ControllerRenderProps,
  FieldValues,
  UseFieldArrayRemove,
} from "react-hook-form";

interface MultiSelectProps {
  field: ControllerRenderProps<FieldValues, any>;
  index: number;
  remove: UseFieldArrayRemove;
  options: { label: string; val: string }[];
  placeholder: string;
  onChange?: (val: string) => void;
}
export default function MultiSelectForm({
  field,
  options,
  placeholder,
  onChange,
}: MultiSelectProps) {
  return (
    <FormItem className="my-2">
      <div className="flex w-full items-center justify-between min-w-16">
        <Select
          onValueChange={(val) => {
            if (onChange) {
              onChange(val);
              return field.onChange(val);
            }
            return field.onChange(val);
          }}
          defaultValue={field.value}
          value={field.value}
        >
          <FormControl>
            <SelectTrigger className="rounded-xl w-full" value={field.value}>
              <SelectValue placeholder={placeholder} />
            </SelectTrigger>
          </FormControl>
          <SelectContent>
            {options.map((l, i) => (
              <SelectItem key={i} value={l.val}>
                {l.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <FormMessage />
    </FormItem>
  );
}
