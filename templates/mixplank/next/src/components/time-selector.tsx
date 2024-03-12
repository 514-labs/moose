"use client"
import { DateRange } from "@/data/time-query";
import { Button } from "./ui/button";

interface Props {
    selectedRange: DateRange,
    setDateRange: (range: DateRange) => void;
}

function DateButton({ selectedRange, setDateRange, range }: Props & { range: DateRange }) {
    return <Button className={`rounded-xl ${selectedRange == range && 'bg-accent'}`} variant={"ghost"} onClick={() => setDateRange(range)}>{range}</Button>

}
export default function TimeSelector(props: Props) {
    return <div className="flex gap-2 mx-4">
        <DateButton {...props} range={DateRange["Today"]} />
        <DateButton {...props} range={DateRange["3D"]} />
        <DateButton {...props} range={DateRange["7D"]} />
        <DateButton {...props} range={DateRange["30D"]} />
    </div>
}