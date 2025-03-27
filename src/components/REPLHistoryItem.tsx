import { useRef, useEffect, useState } from "react";
import { Output } from "../types/Output.type";

type REPLHistoryItemProps = {
    cmd: string,
    output: Output,
    forceMessage: boolean
}

function REPLHistoryItem(props: REPLHistoryItemProps) {
    const [error, setError] = useState(false);
    const date = useRef(new Date());

    useEffect(() => {
        if (props.output.errors && props.output.errors.length !== 0)
            setError(true);
    })

    return (
        <div className={`w-full p-1 ${error ? "bg-orange-950" : "bg-slate-800"} text-white text-xs rounded-md mt-2 grid grid-rows-1 grid-cols-2 max-h-[20vh] overflow-x-auto`}>
            <div>
                <p className="italic text-xs">{props.cmd}</p>
                { props.forceMessage ? (
                    <p className="font-bold border-solid border-t-2 border-slate-600 mt-1">{ props.output.results as string[] }</p>
                ) : (
                    <p className="font-bold border-solid border-t-2 border-slate-600 mt-2">{ error ? props.output.errors.join(" → ") : props.output.results.join(" → ") }</p>
                ) }
            </div>

            <div className="justify-items-end">
                <p className="italic text-slate-600" style={{ fontSize: "0.6rem" }}>{`${date.current.getMonth() + 1} / ${date.current.getDate()} / ${date.current.getFullYear()} @ ${date.current.getHours()}:${date.current.getMinutes()}`}</p>
            </div>
        </div>
    )
}

export default REPLHistoryItem;
