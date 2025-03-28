import AddLayerButton from "./AddLayerButton";
import DBConnectButton from "./DBConnectButton";
import SaveButton from "./SaveButton";
import ServerConfigButton from "./ServerConfigButton";

function ControlBar() {
    return (
        <div className="w-full bg-slate-800 grid grid-rows-1 grid-cols-35">
            <SaveButton />
            <DBConnectButton />
            <AddLayerButton />
            <ServerConfigButton />
        </div>
    )
}

export default ControlBar;
