import { groupBy } from "lodash";
import { MixPanelData, mixpanelData } from "./mixpanel-data";
import { CTAClickEvent, NavClickEvent, PageViewEvent, sendCtaClickEvent, sendNavClickEvent, sendPageViewEvent } from '../../../moose/.moose/ts-sdk'
function marshalEvents(event: MixPanelData) {
    return {
        time: new Date(event.properties.time * 1000),
        distinct_id: event.properties.distinct_id,
        city: event.properties.$city,
        insert_id: event.properties.$insert_id,
        region: event.properties.$region,
        env: event.properties.env,
        host: event.properties.host,
        referer: event.properties.referer,
        ...(event.properties?.href && { href: event.properties.href }),
    }
}
export function loadMixPanelData() {
    console.log("Loading Mixpanel Data")
    const groupedEvents = groupBy(mixpanelData, 'event')
    console.log(sendPageViewEvent)
    const pageViewEvents = groupedEvents.page_view;
    const ctaClickEvents = groupedEvents['cta-click'];
    const navClickEvents = groupedEvents['nav-click'];
    pageViewEvents.map(marshalEvents).forEach(event => {
        sendPageViewEvent(event as PageViewEvent)
    })
    ctaClickEvents.map(marshalEvents).forEach(event => {
        sendCtaClickEvent(event as CTAClickEvent)
    })
    navClickEvents.map(marshalEvents).forEach(event => {
        sendNavClickEvent(event as NavClickEvent)
    })

}