import { useState } from "react";

function DBConnectButton() {
    const [formVisible, setFormVisible] = useState(false);

    return (
        <div className="btn p-2 border border-solid border-slate-700 w-8">
            <div className="z-[999] rounded-md w-1/2 text-white p-5 absolute top-1/3 left-1/2 transform -translate-x-1/3 -translate-y-1/2 bg-slate-950 text-center opacity-90" style={{
                visibility: formVisible ? "visible" : "hidden"
            }}>
                <div className="grid grid-rows-1 grid-cols-[98%_2%]">
                    <h1 className="text-lg pb-3">Connect to a PostGIS Database</h1>

                    <div onClick={ () => setFormVisible(false) }>
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" className="bi bi-x-lg" viewBox="0 0 16 16">
                            <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
                        </svg>
                    </div>
                </div>
                <form className="grid grid-cols-1 grid-rows-3 justify-items-center text-sm" onSubmit={ (event) => {
                    event.preventDefault();
                    const username = (document.getElementById("db-connect-username") as HTMLInputElement).value;
                    const password = (document.getElementById("db-connect-password") as HTMLInputElement).value;
                    const host = (document.getElementById("db-connect-host") as HTMLInputElement).value;
                    const port = (document.getElementById("db-connect-port") as HTMLInputElement).value;
                    const db = (document.getElementById("db-connect-database") as HTMLInputElement).value;
                    const params = (document.getElementById("db-connect-optional-params") as HTMLInputElement).value;

                    (document.getElementById("repl-input") as HTMLTextAreaElement)!.value = `db connect ${username} ${password} ${host} ${port} ${db} ${params}`;
                    (document.getElementById("repl-form") as HTMLFormElement)!.requestSubmit();

                    setFormVisible(false);
                }}>
                    <div>
                        <label htmlFor="db-connect-username" className="text-slate-700">postgresql://</label>
                        <input id="db-connect-username" className="m-1 p-1 w-30 bg-slate-800 text-white border-solid border-2 border-slate-800 rounded-md" placeholder="username" type="text"/>

                        <label htmlFor="db-connect-password" className="text-slate-700">:</label>
                        <input id="db-connect-password" className="m-1 p-1 w-25 bg-slate-800 text-white border-solid border-2 border-slate-800 rounded-md" placeholder="password" type="password"/>

                        <label htmlFor="db-connect-host" className="text-slate-700">@</label>
                        <input id="db-connect-host" className="m-1 p-1 bg-slate-800 text-white border-solid border-2 border-slate-800 rounded-md" placeholder="host" type="text"/>

                        <label htmlFor="db-connect-port" className="text-slate-700">:</label>
                        <input id="db-connect-port" className="m-1 p-1 w-13 bg-slate-800 text-white border-solid border-2 border-slate-800 rounded-md" placeholder="port" type="text"/>

                        <label htmlFor="db-connect-database" className="text-slate-700">/</label>
                        <input id="db-connect-database" className="m-1 p-1 w-25 bg-slate-800 text-white border-solid border-2 border-slate-800 rounded-md" placeholder="database" type="text"/>
                    </div>

                    <input id="db-connect-optional-params" className="p-1 w-[90%] bg-slate-800 text-white border-solid border-2 border-slate-800 rounded-md" placeholder="optional parameters" type="text"/>

                    <input className="mt-5 w-1/4 bg-slate-300 text-black border-solid border-2 border-slate-800 hover:bg-slate-600 hover:text-white" value="Connect" type="submit"/>
                </form>
            </div>

            <svg onClick={ () => setFormVisible(!formVisible) } xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" className="bi bi-database-add text-slate-500" viewBox="0 0 16 16">
                <path d="M12.5 16a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7m.5-5v1h1a.5.5 0 0 1 0 1h-1v1a.5.5 0 0 1-1 0v-1h-1a.5.5 0 0 1 0-1h1v-1a.5.5 0 0 1 1 0"/>
                <path d="M12.096 6.223A5 5 0 0 0 13 5.698V7c0 .289-.213.654-.753 1.007a4.5 4.5 0 0 1 1.753.25V4c0-1.007-.875-1.755-1.904-2.223C11.022 1.289 9.573 1 8 1s-3.022.289-4.096.777C2.875 2.245 2 2.993 2 4v9c0 1.007.875 1.755 1.904 2.223C4.978 15.71 6.427 16 8 16c.536 0 1.058-.034 1.555-.097a4.5 4.5 0 0 1-.813-.927Q8.378 15 8 15c-1.464 0-2.766-.27-3.682-.687C3.356 13.875 3 13.373 3 13v-1.302c.271.202.58.378.904.525C4.978 12.71 6.427 13 8 13h.027a4.6 4.6 0 0 1 0-1H8c-1.464 0-2.766-.27-3.682-.687C3.356 10.875 3 10.373 3 10V8.698c.271.202.58.378.904.525C4.978 9.71 6.427 10 8 10q.393 0 .774-.024a4.5 4.5 0 0 1 1.102-1.132C9.298 8.944 8.666 9 8 9c-1.464 0-2.766-.27-3.682-.687C3.356 7.875 3 7.373 3 7V5.698c.271.202.58.378.904.525C4.978 6.711 6.427 7 8 7s3.022-.289 4.096-.777M3 4c0-.374.356-.875 1.318-1.313C5.234 2.271 6.536 2 8 2s2.766.27 3.682.687C12.644 3.125 13 3.627 13 4c0 .374-.356.875-1.318 1.313C10.766 5.729 9.464 6 8 6s-2.766-.27-3.682-.687C3.356 4.875 3 4.373 3 4"/>
            </svg>
        </div>
    )
}

export default DBConnectButton;
