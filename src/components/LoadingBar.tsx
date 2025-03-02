import { useState } from "react";
import { listen } from "@tauri-apps/api/event";

function LoadingBar() {
    const [loadingPercent, setLoadingPercent] = useState(0);

    listen<string>('loading', (event) => {
        setLoadingPercent(parseInt(event.payload));
    });

    return (
        <div className="p-2 w-full h-[5vh] bg-stone-800 text-stone-600 italic">
        <div className="p-2 h-[1vh] bg-blue-600 rounded-md" style={{
            transition: "width 0.1s ease-in-out",
            WebkitTransition: "width 0.1s ease-in-out",
            width: `${loadingPercent}%`,
            visibility: `${ loadingPercent == 0 ? "hidden" : "visible" }`
        }}></div>
        </div>
    )
}

export default LoadingBar;
