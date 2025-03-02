import { open } from "@tauri-apps/plugin-dialog";

function AddLayerButton() {
    return (
        <div className="btn p-2 border border-solid border-stone-700 w-8" onClick={ async () => {
            const selected = await open({
                multiple: false,
            });

            if (selected) {
                (document.getElementById("repl-input") as HTMLTextAreaElement)!.value = `add layer '${selected}'`;
                (document.getElementById("repl-form") as HTMLFormElement)!.requestSubmit();
            }
        } }>
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" className="bi bi-plus-lg text-stone-500" viewBox="0 0 16 16">
                <path fill-rule="evenodd" d="M8 2a.5.5 0 0 1 .5.5v5h5a.5.5 0 0 1 0 1h-5v5a.5.5 0 0 1-1 0v-5h-5a.5.5 0 0 1 0-1h5v-5A.5.5 0 0 1 8 2"/>
            </svg>
        </div>
    )
}

export default AddLayerButton;
