import { useCallback, useState } from "react";
import { TopLevelSpec } from "vega-lite";
import { Mark } from "vega";
const vl = require("vega-lite-api")




export function deserializeSpec(spec: TopLevelSpec) {
    const configuration = vl.spec(spec)
    console.log(configuration, 'spec')
    const marks = configuration.mark()
    const encoding = configuration.encoding()
    return { marks, encoding }
}

export function useChartBinding() {
    const [spec, setState] = useState(vl.mark('line'));

    const setChartType = useCallback((mark: string) => setState((vl: any) => vl.mark(mark)), [])
    const setChartData = useCallback((data: any) => setState((vl: any) => vl.data(data)), [])
    return {
        spec: spec.toSpec(),
        setChartType,
        setChartData
    }
}