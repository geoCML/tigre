import AddLayerButton from "./AddLayerButton";
import DBConnectButton from "./DBConnectButton";
import SaveButton from "./SaveButton";

function ControlBar() {
    return (
        <div className="w-full bg-stone-800 grid grid-rows-1 grid-cols-35">
            <SaveButton />
            <DBConnectButton />
            <AddLayerButton />
        </div>
    )
}

export default ControlBar;
