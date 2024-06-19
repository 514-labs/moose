import { useFieldArray, useForm } from "react-hook-form";
import { Form, FormControl, FormField, FormItem, FormMessage } from "./ui/form";
import { Button } from "./ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";
import { useEffect } from "react";
import { Plus, XIcon } from "lucide-react";

export type MultiSelectList = { label: string; val: string }[];

interface MultiSelectProps {
  setForm: (form: string[]) => void;
  options: MultiSelectList;
  name: string;
}
export default function MultiSelectForm({
  setForm,
  options,
  name,
}: MultiSelectProps) {
  const form = useForm({ mode: "onChange" });
  const { handleSubmit, watch } = form;

  useEffect(() => {
    const subscription = watch(() =>
      handleSubmit((val) => {
        setForm(val.list);
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
          onClick={() => append(null, { shouldFocus: true })}
        >
          {name}
          <Plus />
        </Button>
        <div className="p-2">
          {fields.map((_field, index) => (
            <FormField
              control={form.control}
              key={`list.${index}`}
              name={`list.${index}`}
              render={({ field }) => (
                <FormItem className="my-2">
                  <div className="flex items-center">
                    <Select
                      onValueChange={(val) => field.onChange(val)}
                      defaultValue={field.value}
                      value={field.value}
                    >
                      <FormControl>
                        <SelectTrigger
                          className="rounded-xl"
                          value={field.value}
                        >
                          <SelectValue placeholder="Select Event" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {options.map((l, i) => (
                          <SelectItem key={i} value={l.val}>
                            {l.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                      <Button
                        className="ml-4 p-2 h-fit"
                        size={"sm"}
                        variant={"ghost"}
                        onClick={() => remove(index)}
                      >
                        <XIcon className="h-6 w-6" />
                      </Button>
                    </Select>
                  </div>
                  <FormMessage />
                </FormItem>
              )}
            />
          ))}
        </div>
      </form>
    </Form>
  );
}
