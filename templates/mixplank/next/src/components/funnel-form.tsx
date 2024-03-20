import { useFieldArray, useForm } from "react-hook-form"
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from "./ui/form";
import { Button } from "./ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";
import { eventNameMap, eventTables } from "@/insights/event-tables";
import { useEffect } from "react";
import { XIcon } from "lucide-react";

export type FunnelFormList = { events: { eventName: string, tableName: string }[] }
interface FunnelFormProps {
    setForm: (form: FunnelFormList) => void;
}
export default function FunnelForm({ setForm }: FunnelFormProps) {
    const form = useForm({ mode: "onChange" });
    const { handleSubmit, watch } = form;

    useEffect(() => {
        // TypeScript users 
        const subscription = watch(() => handleSubmit(setForm)())
        return () => subscription.unsubscribe();
    }, [handleSubmit, watch, setForm]);

    const { fields, append, remove } = useFieldArray({
        name: "events",
        control: form.control,
    })

    return <Form {...form}>
        <form>
            <div>
                {fields.map((field, index) => (
                    <FormField
                        control={form.control}
                        key={field.id}
                        name={`events.${index}`}
                        render={({ field }) => (
                            <FormItem>
                                <FormLabel>Event</FormLabel>
                                <div className="flex">
                                    <Select onValueChange={(val) => field.onChange(eventNameMap[val])} defaultValue={field.value}>
                                        <FormControl>
                                            <SelectTrigger>
                                                <SelectValue placeholder="Select an Event" />
                                            </SelectTrigger>
                                        </FormControl>
                                        <SelectContent>
                                            {eventTables.map((event, i) => (
                                                <SelectItem key={i} value={event.eventName}>{event.eventName}</SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                    <Button className="ml-4" size={"sm"} variant={"ghost"} onClick={() => remove(index)}><XIcon /></Button>
                                </div>
                                <FormMessage />
                            </FormItem>
                        )}
                    />
                ))}
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="mt-2"
                    onClick={() => append({ eventName: null, tableName: null })}
                >
                    Add Event
                </Button>
            </div>
        </form>
    </Form>
}