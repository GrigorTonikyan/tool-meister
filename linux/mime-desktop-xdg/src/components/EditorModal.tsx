import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DesktopFile } from "@/types";
import { Button } from "@/components/ui/button";
import { Check, Edit, X, WandSparkles } from "lucide-react";

interface EditorModalProps {
    file: DesktopFile | null;
    onClose: () => void;
    onRefresh: () => void;
}

export function EditorModal({ file, onClose, onRefresh }: EditorModalProps) {
    const [content, setContent] = useState("");
    const [loading, setLoading] = useState(false);
    const [msg, setMsg] = useState("");

    useEffect(() => {
        if (file) {
            setLoading(true);
            invoke("read_file", { path: file.absolutePath }).then((c: any) => {
                setContent(c);
            }).catch(e => {
                setMsg(`Error reading: ${e}`);
            }).finally(() => setLoading(false));
            
            // Lock body scroll
            document.body.style.overflow = 'hidden';
            return () => {
                document.body.style.overflow = 'unset';
            };
        }
    }, [file]);

    if (!file) return null;

    const handleSave = async () => {
        setLoading(true);
        try {
            await invoke("git_snapshot", { message: `Auto snapshot before editing ${file.filename}` });
            await invoke("save_desktop_file", { path: file.absolutePath, content });
            setMsg("Saved successfully.");
            setTimeout(() => {
                onRefresh();
                onClose();
            }, 1000);
        } catch (e) {
            setMsg(`Save failed: ${e}`);
        }
        setLoading(false);
    };

    const handleAutoFix = async () => {
        setLoading(true);
        try {
            await invoke("git_snapshot", { message: `Auto snapshot before AutoFixing ${file.filename}` });
            const result = await invoke<string>("auto_fix_desktop_file", { absolutePath: file.absolutePath, filename: file.filename });
            setMsg(`Auto Fix Applied: ${result}`);

            // Reload content if it was an in-place content fix
            const c: any = await invoke("read_file", { path: result.includes(".desktop") ? result : file.absolutePath });
            setContent(c);
            setTimeout(() => onRefresh(), 1500);
        } catch (e) {
            setMsg(`Auto Fix failed: ${e}`);
        }
        setLoading(false);
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4 animate-in fade-in duration-300">
            <div className="bg-zinc-950 border border-zinc-800 rounded-2xl shadow-2xl w-full max-w-5xl h-[85vh] flex flex-col overflow-hidden animate-in zoom-in-95 duration-300">
                <div className="flex justify-between items-center p-6 bg-zinc-900/50 border-b border-zinc-800 backdrop-blur-xl">
                    <div className="flex items-center gap-4">
                        <div className="p-3 bg-indigo-500/10 rounded-xl border border-indigo-500/20">
                            <Edit className="w-6 h-6 text-indigo-400" />
                        </div>
                        <div>
                            <h2 className="text-xl font-bold text-zinc-100 tracking-tight">
                                Editing Metadata
                            </h2>
                            <p className="text-sm text-zinc-500 font-mono mt-0.5">{file.absolutePath}</p>
                        </div>
                    </div>
                    <Button variant="ghost" size="icon" onClick={onClose} className="text-zinc-400 hover:text-white hover:bg-zinc-800 rounded-full h-10 w-10">
                        <X className="w-5 h-5" />
                    </Button>
                </div>

                <div className="p-6 flex-1 overflow-hidden flex flex-col gap-4 bg-linear-to-b from-zinc-950 to-black">
                    {msg && (
                        <div className={`p-4 rounded-xl border flex items-center gap-3 animate-in slide-in-from-top-2 ${
                            msg.includes("Error") || msg.includes("failed") 
                            ? "bg-red-500/10 border-red-500/20 text-red-400" 
                            : "bg-emerald-500/10 border-emerald-500/20 text-emerald-400"
                        }`}>
                            <div className="text-sm font-bold">{msg}</div>
                        </div>
                    )}

                    <div className="flex-1 relative group">
                        <div className="absolute inset-0 bg-indigo-500/5 blur-3xl rounded-full opacity-0 group-focus-within:opacity-100 transition-opacity duration-1000"></div>
                        <textarea
                            className="relative w-full h-full p-6 bg-zinc-900/30 border border-zinc-800/80 rounded-2xl text-sm font-mono text-indigo-100/90 focus:ring-1 focus:ring-indigo-500/50 focus:border-indigo-500/50 outline-none resize-none leading-relaxed transition-all shadow-inner custom-scrollbar"
                            value={content}
                            onChange={(e) => setContent(e.target.value)}
                            disabled={loading}
                            spellCheck={false}
                        />
                    </div>
                </div>

                <div className="p-6 border-t border-zinc-800/50 flex justify-between items-center bg-zinc-900/50 backdrop-blur-xl">
                    <Button 
                        variant="outline" 
                        onClick={handleAutoFix} 
                        disabled={loading}
                        className="bg-amber-500/10 text-amber-500 border-amber-500/20 hover:bg-amber-500/20 hover:text-amber-400 rounded-xl h-11 px-6 font-bold transition-all"
                    >
                        <WandSparkles className="w-4 h-4 mr-2" /> Magic Repair Content
                    </Button>
                    <div className="flex gap-3">
                        <Button 
                            variant="ghost" 
                            onClick={onClose} 
                            className="text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800/50 rounded-xl h-11 px-6 font-bold"
                        >
                            Discard
                        </Button>
                        <Button 
                            onClick={handleSave} 
                            disabled={loading}
                            className="bg-indigo-600 hover:bg-indigo-700 text-white shadow-lg shadow-indigo-600/20 rounded-xl h-11 px-8 font-bold transition-all"
                        >
                            {loading ? <X className="w-4 h-4 mr-2 animate-spin" /> : <Check className="w-4 h-4 mr-2" />}
                            Write Changes
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    );
}
