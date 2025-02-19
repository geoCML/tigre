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
        <div className={`w-full p-2 ${error ? "bg-orange-950" : "bg-stone-800"} text-white rounded-md mt-5 mb-5 grid grid-rows-1 grid-cols-2 max-h-[20vh] overflow-y-auto`}>
            <div>
                <p className="italic text-sm">{props.cmd}</p>
                { props.forceMessage ? (
                    <p className="font-bold">{ props.output.results as string[] }</p>
                ) : (
                    <p className="font-bold">{ error ? props.output.errors.join(" → ") : props.output.results.join(" → ") }</p>
                ) }
            </div>

            <div className="grid grid-cols-1 grid-rows-1 justify-items-end">
                <p className="italic text-sm text-stone-600">{`${date.current.getMonth()} / ${date.current.getDay()} / ${date.current.getFullYear()} @ ${date.current.getHours()}:${date.current.getMinutes()}`}</p>
            </div>
        </div>
    )
}

export default REPLHistoryItem;
