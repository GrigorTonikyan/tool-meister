import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { GitSnapshot } from "@/types";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { GitBranch, GitCommit, LogOut, Clock, RotateCcw, Eye } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";

interface SnapshotViewerProps {
    onViewDiff: (id: string) => void;
}

export function SnapshotViewer({ onViewDiff }: SnapshotViewerProps) {
    const [snapshots, setSnapshots] = useState<GitSnapshot[]>([]);
    const [loading, setLoading] = useState(false);
    const [alert, setAlert] = useState<{type: 'success' | 'error', message: string} | null>(null);

    const fetchSnapshots = async () => {
        setLoading(true);
        try {
            const res = await invoke<GitSnapshot[]>("list_snapshots");
            setSnapshots(res);
        } catch (err) {
            setAlert({ type: 'error', message: `History error: ${err}` });
        }
        setLoading(false);
    };

    useEffect(() => {
        fetchSnapshots();
    }, []);

    const handleRestoreCommit = async (id: string) => {
        if (!confirm("Are you sure you want to revert to this specific snapshot? ALL CURRENT UNSAVED CHANGES WILL BE LOST.")) return;
        setLoading(true);
        try {
            await invoke("git_restore_to_commit", { commitId: id });
            setAlert({ type: "success", message: "Successfully reverted to historical snapshot!" });
            fetchSnapshots();
        } catch (err) {
            setAlert({ type: "error", message: `Restore failed: ${err}` });
        }
        setLoading(false);
    };

    return (
        <div className="flex flex-col gap-6 animate-in fade-in duration-500">
            <div className="flex justify-between items-center bg-zinc-900/40 p-6 rounded-2xl border border-zinc-800/50 backdrop-blur-md">
                <div className="flex items-center gap-4">
                    <div className="p-3 bg-indigo-500/10 rounded-xl border border-indigo-500/20">
                        <GitBranch className="w-6 h-6 text-indigo-400" />
                    </div>
                    <div>
                        <h2 className="text-xl font-bold text-zinc-100">Configuration History</h2>
                        <p className="text-sm text-zinc-400">Manage snapshots of your desktop environment entries</p>
                    </div>
                </div>
                <Button 
                    variant="outline" 
                    onClick={fetchSnapshots} 
                    disabled={loading}
                    className="border-zinc-800 bg-zinc-900 hover:bg-zinc-800 h-10 px-6 rounded-xl transition-all"
                >
                    <RotateCcw className={`w-4 h-4 mr-2 ${loading ? 'animate-spin' : ''}`} /> Refresh Timeline
                </Button>
            </div>

            {alert && (
                <Alert className={`${alert.type === 'error' ? 'bg-red-900/20 border-red-900/30' : 'bg-emerald-900/20 border-emerald-900/30'} border animate-in slide-in-from-top-4`}>
                    <AlertTitle className="capitalize font-bold">{alert.type}</AlertTitle>
                    <AlertDescription>{alert.message}</AlertDescription>
                </Alert>
            )}

            <div className="relative pl-10 border-l border-zinc-800/50 space-y-8 py-4 ml-4">
                {snapshots.length === 0 && !loading && (
                    <div className="flex flex-col items-center justify-center py-20 bg-zinc-900/20 rounded-3xl border border-dashed border-zinc-800">
                        <LogOut className="w-12 h-12 text-zinc-700 mb-4" />
                        <p className="text-zinc-500 italic">No historical snapshots detected.</p>
                        <p className="text-zinc-600 text-xs mt-2">Initialize a repository to start tracking changes.</p>
                    </div>
                )}
                
                {snapshots.map((snap, idx) => (
                    <div key={snap.commit_id} className="relative group">
                        {/* Status Indicator Connector */}
                        <div className={`absolute -left-11.25 top-2 w-2.5 h-2.5 rounded-full border-2 border-zinc-950 transition-all duration-300 ${
                            idx === 0 ? 'bg-emerald-500 shadow-[0_0_12px_rgba(16,185,129,0.6)] scale-125' : 'bg-indigo-500 group-hover:scale-110'
                        }`} />
                        
                        <div className="bg-zinc-900/40 backdrop-blur-xl border border-zinc-800/50 rounded-2xl p-6 hover:border-zinc-600/50 transition-all hover:bg-zinc-900/60 shadow-lg shadow-black/20">
                            <div className="flex justify-between items-start mb-3">
                                <div className="flex items-center gap-3">
                                    <h3 className="text-base font-bold text-zinc-100 group-hover:text-indigo-400 transition-colors">{snap.message || "Manual Snapshot"}</h3>
                                    {idx === 0 && <Badge className="bg-emerald-500/10 text-emerald-400 border-emerald-500/20 px-2.5 py-0.5 rounded-lg text-[10px] h-auto">LATEST</Badge>}
                                </div>
                                <span className="text-[10px] font-mono text-zinc-500 bg-zinc-950/80 border border-zinc-800 px-2 py-1 rounded-md">
                                    {snap.commit_id.substring(0, 8)}
                                </span>
                            </div>
                            
                            <div className="flex flex-wrap items-center gap-6 text-xs text-zinc-500 mb-6">
                                <div className="flex items-center gap-2">
                                    <Clock className="w-3.5 h-3.5 text-zinc-600" /> 
                                    {new Date(snap.timestamp * 1000).toLocaleString()}
                                </div>
                                <div className="flex items-center gap-2">
                                    <GitCommit className="w-3.5 h-3.5 text-zinc-600" /> 
                                    <span className="text-zinc-400 font-medium">{snap.author}</span>
                                </div>
                            </div>

                            <div className="flex justify-end gap-3 pt-4 border-t border-zinc-800/50">
                                <Button 
                                    size="sm" 
                                    variant="ghost"
                                    onClick={() => onViewDiff(snap.commit_id)}
                                    className="h-10 text-[11px] font-bold uppercase tracking-widest text-zinc-400 hover:text-indigo-400 hover:bg-indigo-500/10 rounded-xl px-5 transition-all"
                                >
                                    <Eye className="w-4 h-4 mr-2" /> View Changes
                                </Button>
                                <Button 
                                    size="sm" 
                                    variant="secondary"
                                    onClick={() => handleRestoreCommit(snap.commit_id)}
                                    disabled={loading || idx === 0}
                                    className="h-10 text-[11px] font-bold uppercase tracking-widest bg-zinc-800/80 hover:bg-indigo-600 hover:text-white transition-all disabled:opacity-20 rounded-xl px-5 border border-zinc-700/50"
                                >
                                    <RotateCcw className="w-3 h-3 mr-2" /> Restore State
                                </Button>
                            </div>
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
}
