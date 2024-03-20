import FunnelForm, { FunnelFormList } from "./funnel-form";

interface Props {
    setForm: (form: FunnelFormList) => void;
    form: FunnelFormList;
}
export default function TimeSeriesForm({ setForm, form }: Props) {
    const cleanedEvents = form.events.filter((ev) => ev.eventName != null)
    return (<div>
        {!cleanedEvents.length && <div>Displaying all events</div>}
        <FunnelForm setForm={setForm} />
    </div>)
}