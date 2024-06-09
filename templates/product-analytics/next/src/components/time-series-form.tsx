import { eventTables } from "@/app/events";
import MultiSelectForm from "./multi-select-form";
import { ModelMeta } from "@/insights/model-meta";
import { MetricForm } from "@/lib/form-types";
import MetricSelectForm from "./metric-form";

interface Props {
  setForm: (form: MetricForm[]) => void;
  setBreakdown: (form: string[]) => void;
  breakdownOptions: ModelMeta[];
}
export default function TimeSeriesForm({
  setForm,
  breakdownOptions,
  setBreakdown,
}: Props) {
  const list = eventTables.map((table) => ({
    label: table.eventName,
    val: table.tableName,
  }));

  const breakdownList = breakdownOptions.map((bd) => ({
    label: bd.name,
    val: bd.name,
  }));

  return (
    <div>
      <MetricSelectForm options={list} setForm={setForm} name="Metrics" />
      <MultiSelectForm
        options={breakdownList}
        setForm={setBreakdown}
        name="Breakdown"
      />
    </div>
  );
}
