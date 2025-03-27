import { useState } from "react";
import { listen } from "@tauri-apps/api/event";

function LoadingBar() {
    const [loadingPercent, setLoadingPercent] = useState(0);

    listen<string>('loading', (event) => {
        setLoadingPercent(parseInt(event.payload));
    });

    return (
        <div className="p-2 w-full h-[3vh] bg-slate-800 text-slate-600 italic">
        <div className="p-1 h-[2px] bg-blue-600 rounded-md" style={{
            transition: "width 0.1s ease-in-out",
            WebkitTransition: "width 0.1s ease-in-out",
            width: `${loadingPercent}%`,
            visibility: `${ loadingPercent == 0 ? "hidden" : "visible" }`
        }}></div>
        </div>
    )
}

export default LoadingBar;
