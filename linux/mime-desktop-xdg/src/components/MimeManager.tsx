import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MimeAssociation } from "@/types";
import { Input } from "@/components/ui/input";
import { Settings, Search, AppWindow, ExternalLink, Filter, AlertCircle } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";

export function MimeManager() {
    const [associations, setAssociations] = useState<MimeAssociation[]>([]);
    const [loading, setLoading] = useState(false);
    const [searchTerm, setSearchTerm] = useState("");
    const [status, setStatus] = useState<{ type: 'success' | 'error', message: string } | null>(null);

    const fetchMime = async () => {
        setLoading(true);
        try {
            const res = await invoke<MimeAssociation[]>("get_mime_associations");
            setAssociations(res);
        } catch (err) {
            setStatus({ type: 'error', message: `MIME error: ${String(err)}` });
        }
        setLoading(false);
    };

    useEffect(() => {
        fetchMime();
    }, []);

    const handleSetDefault = async (mime: string, app: string) => {
        setLoading(true);
        try {
            await invoke("git_snapshot", { message: `Change default for ${mime} to ${app}` });
            await invoke("set_mime_association", { mimeType: mime, desktopFile: app });
            setStatus({ type: 'success', message: `Successfully updated ${mime} default!` });
            fetchMime();
        } catch (err) {
            setStatus({ type: 'error', message: `Update failed: ${String(err)}` });
        }
        setLoading(false);
    };

    const filtered = associations.filter(a => 
        a.mimeType.toLowerCase().includes(searchTerm.toLowerCase()) ||
        a.defaultApps.some(app => app.toLowerCase().includes(searchTerm.toLowerCase()))
    );

    return (
        <div className="flex flex-col gap-6 animate-in fade-in duration-700 h-full">
            <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-zinc-900/60 p-6 rounded-2xl border border-zinc-800/50 backdrop-blur-xl shadow-2xl">
                <div className="flex items-center gap-4">
                    <div className="p-3 bg-blue-500/10 rounded-xl border border-blue-500/20">
                        <Settings className="w-6 h-6 text-blue-400" />
                    </div>
                    <div>
                        <h2 className="text-xl font-bold text-zinc-100 italic tracking-tight">System MIME Associations</h2>
                        <p className="text-sm text-zinc-400">Manage default application handlers in ~/.config/mimeapps.list</p>
                    </div>
                </div>
                
                <div className="relative w-full md:w-80 group">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-500 group-focus-within:text-blue-400 transition-colors" />
                    <Input 
                        placeholder="Search type or app..." 
                        className="pl-10 bg-zinc-950 border-zinc-800 focus:border-blue-500 rounded-xl py-5 transition-all outline-none"
                        value={searchTerm}
                        onChange={e => setSearchTerm(e.target.value)}
                    />
                </div>
            </div>

            {status && (
                <Alert className={`${status.type === 'error' ? 'bg-red-900/20 border-red-900/40' : 'bg-blue-900/20 border-blue-900/40'} border animate-in slide-in-from-right-4 transition-all`}>
                    <AlertCircle className="h-4 w-4" />
                    <AlertTitle className="capitalize font-bold">{status.type}</AlertTitle>
                    <AlertDescription>{status.message}</AlertDescription>
                </Alert>
            )}

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 h-full overflow-y-auto pr-2 custom-scrollbar pb-10">
                {filtered.map(assoc => (
                    <div key={assoc.mimeType} className="flex flex-col p-5 bg-zinc-900/40 border border-zinc-800/80 rounded-2xl hover:border-blue-500/30 transition-all group hover:bg-zinc-900/60 hover:shadow-xl hover:shadow-blue-500/5">
                        <div className="flex justify-between items-start mb-4">
                            <div className="flex items-center gap-3">
                                <div className="p-2 bg-zinc-800 rounded-lg group-hover:bg-blue-900/20 transition-colors">
                                    <AppWindow className="w-4 h-4 text-zinc-400 group-hover:text-blue-400" />
                                </div>
                                <div>
                                    <h3 className="text-sm font-bold text-zinc-200 truncate max-w-50" title={assoc.mimeType}>
                                        {assoc.mimeType}
                                    </h3>
                                    <Badge variant="secondary" className="mt-1 bg-zinc-950 text-zinc-500 border-zinc-800 font-mono text-[10px] h-auto py-0 px-1.5">
                                        Type
                                    </Badge>
                                </div>
                            </div>
                            <Badge className="bg-blue-500/10 text-blue-400 border-blue-500/20 px-2.5 py-1 rounded-lg text-[10px]">
                                ACTIVE
                            </Badge>
                        </div>

                        <div className="flex flex-col gap-3 mt-auto">
                            <div className="flex flex-col gap-1.5">
                                <label className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest pl-1">Default Handler</label>
                                <div className="flex items-center justify-between p-3 bg-zinc-950/80 border border-zinc-800 rounded-xl group-hover:border-zinc-700 transition-colors">
                                    <span className="text-xs text-blue-300 font-mono truncate">{assoc.defaultApps[0] || "None"}</span>
                                    <ExternalLink className="w-3 h-3 text-zinc-600" />
                                </div>
                            </div>
                            
                            {assoc.defaultApps.length > 1 && (
                                <div className="flex flex-col gap-2 pt-2 border-t border-zinc-800/50 mt-2">
                                    <label className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest pl-1 flex items-center gap-2">
                                        <Filter className="w-3 h-3" /> Recommended Alternates
                                    </label>
                                    <div className="flex flex-wrap gap-2">
                                        {assoc.defaultApps.slice(1, 4).map(app => (
                                            <button
                                                key={app}
                                                onClick={() => handleSetDefault(assoc.mimeType, app)}
                                                disabled={loading}
                                                className="text-[10px] bg-zinc-900 hover:bg-blue-600 hover:text-white border border-zinc-800 text-zinc-400 py-1.5 px-3 rounded-lg transition-all"
                                            >
                                                Use {app.replace('.desktop', '')}
                                            </button>
                                        ))}
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                ))}
            </div>
            {filtered.length === 0 && (
                <div className="flex flex-col items-center justify-center py-20 grayscale opacity-40">
                    <Search className="w-16 h-16 mb-4" />
                    <p className="text-zinc-400 font-bold">No MIME types match your search</p>
                </div>
            )}
        </div>
    );
}
