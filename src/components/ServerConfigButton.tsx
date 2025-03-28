import { useState } from "react";

export function ServerConfigButton() { 
    const [formVisible, setFormVisible] = useState(false);

    return (
        <div className="btn p-2 border border-solid border-slate-700 w-8">
            <div className="z-[999] rounded-md w-1/3 h-[57vh] text-white p-5 absolute top-1/2 left-1/2 transform -translate-x-1/3 -translate-y-1/2 bg-slate-950 text-center opacity-90" style={{
                visibility: formVisible ? "visible" : "hidden"
            }}>
                <div className="grid grid-rows-1 grid-cols-[98%_2%]">
                    <h1 className="text-lg pb-3">Configure HyTigre Server</h1>

                    <div onClick={ () => setFormVisible(false) }>
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" className="bi bi-x-lg" viewBox="0 0 16 16">
                            <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
                        </svg>
                    </div>
                </div>
                <form className="grid grid-cols-1 grid-rows-2 justify-items-center text-sm" onSubmit={ (event) => {
                    event.preventDefault();
                    const description = (document.getElementById("server-description") as HTMLInputElement).value;
                    const contactEmail = (document.getElementById("server-contact-email") as HTMLInputElement).value;
                    const contactPhone = (document.getElementById("server-contact-phone") as HTMLInputElement).value;
                    const contactWebsite = (document.getElementById("server-contact-website") as HTMLInputElement).value;

                    (document.getElementById("repl-input") as HTMLTextAreaElement)!.value = `db describe '${description}' ${contactEmail} ${contactPhone} ${contactWebsite}`;
                    (document.getElementById("repl-form") as HTMLFormElement)!.requestSubmit();

                    setFormVisible(false);
                }}>
                    <div className="grid grid-cols-1 grid-rows-auto">
                        <label htmlFor="server-name" className="text-slate-700 mt-2">Server Name</label>
                        <input id="server-name" className="m-1 p-1 w-[25vw] bg-slate-800 text-white border-solid border-2 border-slate-800 rounded-md" placeholder="My Server" type="text" /> 

                        <label htmlFor="server-description" className="text-slate-700 mt-4">Description</label>
                        <textarea id="server-description" className="resize-none m-1 p-1 w-[25vw] h-[10vh] bg-slate-800 text-white border-solid border-2 border-slate-800 rounded-md" placeholder="A long form description of the server and that data it hosts." /> 

                        <label htmlFor="server-contact-email" className="text-slate-700 mt-4">Contact Email</label>
                        <input id="server-contact-email" className="m-1 p-1 w-[25vw] bg-slate-800 text-white border-solid border-2 border-slate-800 rounded-md" placeholder="me@example.com" type="email" /> 

                        <label htmlFor="server-contact-phone" className="text-slate-700 mt-4">Contact Phone</label>
                        <input id="server-contact-phone" className="m-1 p-1 w-[25vw] bg-slate-800 text-white border-solid border-2 border-slate-800 rounded-md" placeholder="123-456-7890" type="tel" /> 

                        <label htmlFor="server-contact-website" className="text-slate-700 mt-4">Contact Website</label>
                        <input id="server-contact-website" className="m-1 p-1 w-[25vw] bg-slate-800 text-white border-solid border-2 border-slate-800 rounded-md" placeholder="https://example.com" type="url" /> 
                    </div>
                    <input className="w-1/4 h-[25px] mt-5 bg-slate-300 text-black border-solid border-2 border-slate-800 hover:bg-slate-600 hover:text-white" value="Save Configuration" type="submit"/> 
                </form> 
            </div>

            <svg onClick={ () => setFormVisible(!formVisible) } xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" className="bi bi-hdd-network text-slate-500" viewBox="0 0 16 16">
                <path d="M4.5 5a.5.5 0 1 0 0-1 .5.5 0 0 0 0 1M3 4.5a.5.5 0 1 1-1 0 .5.5 0 0 1 1 0" />
                <path d="M0 4a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v1a2 2 0 0 1-2 2H8.5v3a1.5 1.5 0 0 1 1.5 1.5h5.5a.5.5 0 0 1 0 1H10A1.5 1.5 0 0 1 8.5 14h-1A1.5 1.5 0 0 1 6 12.5H.5a.5.5 0 0 1 0-1H6A1.5 1.5 0 0 1 7.5 10V7H2a2 2 0 0 1-2-2zm1 0v1a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V4a1 1 0 0 0-1-1H2a1 1 0 0 0-1 1m6 7.5v1a.5.5 0 0 0 .5.5h1a.5.5 0 0 0 .5-.5v-1a.5.5 0 0 0-.5-.5h-1a.5.5 0 0 0-.5.5" />
            </svg>
        </div>
    )
}

export default ServerConfigButton;