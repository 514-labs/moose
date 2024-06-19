import { useFieldArray, useForm } from "react-hook-form";
import { Form } from "../ui/form";
import { Button } from "../ui/button";
import { useEffect } from "react";
import { Plus } from "lucide-react";
import MetricSelectForm from "./metric-select-form";

export interface QueryFormData {
  metricName: string;
  filter: { property: string; value: string; operator: string }[];
  grouping: { property: string }[];
}

export type MultiSelectList = { label: string; val: string }[];

interface MultiSelectProps {
  setForm: (form: QueryFormData) => void;
  options: MultiSelectList;
  name: string;
}
export default function QueryForm({
  setForm,
  options,
  name,
}: MultiSelectProps) {
  const form = useForm({ mode: "onChange" });
  const { handleSubmit, watch } = form;

  useEffect(() => {
    const subscription = watch(() =>
      handleSubmit((val) => {
        if (val.list) {
          setForm(val.list);
        }
      })(),
    );
    return () => subscription.unsubscribe();
  }, [handleSubmit, watch, setForm]);

  const { fields, append, remove } = useFieldArray({
    name: "list",
    control: form.control,
  });

  return (
    <Form {...form}>
      <form className="">
        <Button
          type="button"
          variant="ghost"
          size="default"
          className="mt-2 rounded-xl  w-full justify-between"
          onClick={() =>
            append({ metricName: null, filter: [] }, { shouldFocus: true })
          }
        >
          {name}
          <Plus />
        </Button>
        <div className="p-2">
          {fields.map((_field, index) => {
            return (
              <MetricSelectForm
                key={index}
                form={form}
                index={index}
                remove={remove}
                options={options}
              />
            );
          })}
        </div>
      </form>
    </Form>
  );
}
