import { useEffect, useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DesktopFile, ValidationResult } from "@/types";
import { DiffViewer } from "@/components/DiffViewer";
import { EditorModal } from "@/components/EditorModal";
import { WizardModal } from "@/components/WizardModal";
import { MimeManager } from "@/components/MimeManager";
import { SnapshotViewer } from "@/components/SnapshotViewer";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import {
    Camera,
    Trash2,
    Eye,
    RefreshCw,
    Info,
    ArrowUpDown,
    ArrowUp,
    ArrowDown,
    CopyPlus,
    Edit2,
    Link,
    Clock,
    LayoutGrid,
    Wand2,
    WandSparkles,
    ShieldCheck,
    Check,
    CircleAlert,
    ShieldAlert
} from "lucide-react";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { useCallback, useRef } from "react";

export default function App() {
    const [files, setFiles] = useState<DesktopFile[]>([]);
    const [validations, setValidations] = useState<Record<string, ValidationResult>>({});
    const [loading, setLoading] = useState(true);
    
    // UI State
    const [viewFile, setViewFile] = useState<string | null>(null);
    const [viewDiff, setViewDiff] = useState<string | null>(null);
    const [editFile, setEditFile] = useState<DesktopFile | null>(null);
    const [showWizard, setShowWizard] = useState(false);
    
    const [, setGitStatus] = useState<boolean>(false);
    const [statusMsg, setStatusMsg] = useState("");


    type SortKey = "filename" | "absolutePath" | "locationType" | "isShadowed" | "isValid" | "execExists";
    const [sortConfig, setSortConfig] = useState<{ key: SortKey; direction: "asc" | "desc" } | null>(null);

    // Resizing logic
    const [columnWidths, setColumnWidths] = useState<Record<string, number>>({
        filename: 250,
        absolutePath: 350,
        locationType: 120,
        isShadowed: 120,
        isValid: 120,
        execExists: 150
    });

    const isResizing = useRef<string | null>(null);
    const startX = useRef<number>(0);
    const startWidth = useRef<number>(0);

    const handleMouseDown = useCallback((key: string, e: React.MouseEvent) => {
        isResizing.current = key;
        startX.current = e.clientX;
        startWidth.current = columnWidths[key] || 100;
        
        document.body.style.cursor = 'col-resize';
        document.body.style.userSelect = 'none';

        const onMouseMove = (moveEvent: MouseEvent) => {
            if (!isResizing.current) return;
            const delta = moveEvent.clientX - startX.current;
            const newWidth = Math.max(50, startWidth.current + delta);
            setColumnWidths(prev => ({
                ...prev,
                [isResizing.current!]: newWidth
            }));
        };

        const onMouseUp = () => {
            isResizing.current = null;
            document.body.style.cursor = 'default';
            document.body.style.userSelect = 'auto';
            window.removeEventListener('mousemove', onMouseMove);
            window.removeEventListener('mouseup', onMouseUp);
        };

        window.addEventListener('mousemove', onMouseMove);
        window.addEventListener('mouseup', onMouseUp);
    }, [columnWidths]);

    const handleSort = (key: SortKey) => {
        let direction: "asc" | "desc" = "asc";
        if (sortConfig && sortConfig.key === key && sortConfig.direction === "asc") {
            direction = "desc";
        }
        setSortConfig({ key, direction });
    };

    const refreshData = async () => {
        setLoading(true);
        setStatusMsg("");
        try {
            const scanned: DesktopFile[] = await invoke("scan_desktop_files");
            setFiles(scanned);

            const valMap: Record<string, ValidationResult> = {};
            await Promise.all(
                scanned.map(async (f) => {
                    try {
                        const res: ValidationResult = await invoke("validate_desktop_file", {
                            absolutePath: f.absolutePath,
                        });
                        valMap[f.absolutePath] = res;
                    } catch (e) {
                        console.error("Validation error for", f.absolutePath, e);
                    }
                })
            );
            setValidations(valMap);

            const hasGit = await invoke<boolean>("check_git_status");
            if (!hasGit) {
                await invoke("git_init");
                setGitStatus(true);
            } else {
                setGitStatus(hasGit);
            }
        } catch (e) {
            console.error(e);
            setStatusMsg(`Error loading data: ${e}`);
        }
        setLoading(false);
    };

    useEffect(() => {
        refreshData();
    }, []);

    const handleSnapshot = async () => {
        try {
            await invoke("git_snapshot", { message: `Manual snapshot at ${new Date().toISOString()}` });
            setStatusMsg("Snapshot created successfully.");
        } catch (e) {
            setStatusMsg(`Snapshot failed: ${e}`);
        }
    };

    const handleDelete = async (file: DesktopFile) => {
        if (!confirm(`Are you sure you want to delete ${file.filename}?`)) return;
        try {
            await invoke("git_snapshot", { message: `Auto snapshot before deleting ${file.filename}` });
            await invoke("delete_file", { path: file.absolutePath });
            setStatusMsg(`Successfully deleted ${file.filename}.`);
            refreshData();
        } catch (e) {
            setStatusMsg(`Delete failed: ${e}`);
        }
    };
    const handleAutoFix = async (file: DesktopFile) => {
        setLoading(true);
        try {
            await invoke("git_snapshot", { message: `Auto-fix ${file.filename}` });
            const result = await invoke<string>("auto_fix_desktop_file", { 
                absolutePath: file.absolutePath,
                filename: file.filename
            });
            setStatusMsg(`Fixed: ${result}`);
            refreshData();
        } catch (err) {
            setStatusMsg(`Fix failed: ${String(err)}`);
        }
        setLoading(false);
    };

    const handleFixAll = async (type: "duplicates" | "broken") => {
        const filesToFix = files.filter(f => 
            type === "duplicates" ? f.duplicatePaths?.length > 0 : !validations[f.absolutePath]?.isValid || f.hasBadPattern
        );
        
        if (filesToFix.length === 0) return;
        
        setLoading(true);
        setStatusMsg(`Fixing ${filesToFix.length} files...`);
        try {
            await invoke("git_snapshot", { message: `Bulk auto-fix ${type}` });
            
            let successCount = 0;
            let failureCount = 0;
            let lastError = "";

            for (const file of filesToFix) {
                try {
                    await invoke("auto_fix_desktop_file", { 
                        absolutePath: file.absolutePath,
                        filename: file.filename
                    });
                    successCount++;
                } catch (e: any) {
                    failureCount++;
                    lastError = String(e);
                    console.error("Failed to fix", file.filename, e);
                }
            }
            
            if (failureCount === 0) {
                setStatusMsg(`Successfully fixed ${successCount} issues.`);
            } else if (successCount > 0) {
                setStatusMsg(`Partial success: Fixed ${successCount} files, but ${failureCount} failed. Last error: ${lastError}`);
            } else {
                setStatusMsg(`Bulk fix failed for all ${failureCount} files. Last error: ${lastError}`);
            }
            refreshData();
        } catch (err) {
            setStatusMsg(`Bulk snapshot or init failure: ${String(err)}`);
        }
        setLoading(false);
    };

    const renderTable = (filterType?: string) => {
        let filteredFiles = filterType
            ? files.filter((f) => f.locationType === filterType || (filterType === "system" && (f.locationType === "system" || f.locationType === "systemLocal")))
            : files;
            
        // Additional pure filters
        if (filterType === "duplicates") {
            filteredFiles = files.filter(f => f.duplicatePaths?.length > 0);
        }
        if (filterType === "broken") {
            filteredFiles = files.filter(f => !validations[f.absolutePath]?.execExists || !validations[f.absolutePath]?.isValid || f.hasBadPattern);
        }

        const sortedFiles = useMemo(() => {
            let sortable = [...filteredFiles];
            if (sortConfig !== null) {
                sortable.sort((a, b) => {
                    let aVal: any = a[sortConfig.key as keyof DesktopFile];
                    let bVal: any = b[sortConfig.key as keyof DesktopFile];

                    if (sortConfig.key === "isValid") {
                        aVal = validations[a.absolutePath]?.isValid ?? false;
                        bVal = validations[b.absolutePath]?.isValid ?? false;
                    } else if (sortConfig.key === "execExists") {
                        aVal = validations[a.absolutePath]?.execExists ?? false;
                        bVal = validations[b.absolutePath]?.execExists ?? false;
                    }

                    if (aVal < bVal) return sortConfig.direction === "asc" ? -1 : 1;
                    if (aVal > bVal) return sortConfig.direction === "asc" ? 1 : -1;
                    return 0;
                });
            }
            return sortable;
        }, [filteredFiles, sortConfig, validations]);

            const renderTH = (label: string, key: SortKey) => (
        <TableHead 
            className="p-0 border-r border-zinc-800/50 last:border-r-0 hover:bg-zinc-800/50 transition-colors group select-none relative"
            style={{ width: columnWidths[key] }}
        >
            <div className="flex items-center justify-between h-full p-4">
                <div 
                    className="flex items-center gap-2 cursor-pointer flex-1"
                    onClick={() => handleSort(key)}
                >
                    <span className="font-bold text-zinc-400 group-hover:text-indigo-400 truncate">{label}</span>
                    {sortConfig?.key === key ? (
                        sortConfig.direction === "asc" ? <ArrowUp className="w-3 h-3 text-indigo-400" /> : <ArrowDown className="w-3 h-3 text-indigo-400" />
                    ) : <ArrowUpDown className="w-3 h-3 opacity-30" />}
                </div>
                
                <div 
                    className="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-indigo-500/50 active:bg-indigo-500 transition-colors z-10"
                    onMouseDown={(e) => {
                        e.stopPropagation();
                        handleMouseDown(key, e);
                    }}
                />
            </div>
        </TableHead>
    );

        return (
            <div className="rounded-md border border-zinc-800/50 backdrop-blur-sm bg-zinc-950/50 overflow-hidden">
                <Table className="min-w-full border-collapse" style={{ width: 'max-content', tableLayout: 'auto' }}>
                    <TableHeader className="bg-zinc-900 border-b border-zinc-800 sticky top-0 z-30">
                        <TableRow className="hover:bg-transparent">
                            {renderTH("Filename", "filename")}
                            {renderTH("Path", "absolutePath")}
                            {renderTH("Type", "locationType")}
                            {renderTH("Shadows", "isShadowed")}
                            {renderTH("Format", "isValid")}
                            {renderTH("Binary", "execExists")}
                            <TableHead className="text-right p-4 border-l border-zinc-800/50 sticky right-0 bg-zinc-900 z-40 shadow-[-10px_0_15px_rgba(0,0,0,0.5)]">Actions</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {sortedFiles.map((file) => {
                        const val = validations[file.absolutePath];
                        return (
                            <TableRow key={file.absolutePath}>
                                <TableCell className="font-medium text-zinc-100">
                                    <div className="flex flex-col">
                                        <span>{file.filename}</span>
                                        {file.hasBadPattern && <span className="text-xs text-amber-500 font-bold">Bad Formatting</span>}
                                    </div>
                                </TableCell>
                                <TableCell className="text-xs text-zinc-400 truncate max-w-50" title={file.absolutePath}>
                                    {file.absolutePath}
                                </TableCell>
                                <TableCell>
                                    <Badge variant="outline" className="capitalize">
                                        {file.locationType}
                                    </Badge>
                                </TableCell>
                                <TableCell>
                                    <div className="flex flex-col gap-1">
                                        {file.shadowsPaths?.length > 0 && (
                                            <Badge variant="destructive" title={file.shadowsPaths.join("\n")}>Shadows Others</Badge>
                                        )}
                                        {file.isShadowed && (
                                            <Badge variant="secondary">Is Shadowed</Badge>
                                        )}
                                        {file.duplicatePaths?.length > 0 && (
                                            <Badge variant="default" className="bg-orange-600">Pure Duplicate</Badge>
                                        )}
                                        {!(file.shadowsPaths?.length > 0) && !file.isShadowed && !(file.duplicatePaths?.length > 0) && "-"}
                                    </div>
                                </TableCell>
                                <TableCell>
                                    {val?.isValid ? (
                                        <div className="flex items-center gap-1.5 text-emerald-400 font-bold bg-emerald-500/10 px-2 py-1 rounded-lg border border-emerald-500/20">
                                            <ShieldCheck className="w-3.5 h-3.5" /> Valid
                                        </div>
                                    ) : (
                                        <Popover>
                                            <PopoverTrigger asChild>
                                                <div className="flex items-center gap-1.5 text-red-400 font-bold bg-red-500/10 px-2 py-1 rounded-lg border border-red-500/20 cursor-pointer hover:bg-red-500/20 transition-colors">
                                                    <CircleAlert className="w-3.5 h-3.5" /> Broken
                                                </div>
                                            </PopoverTrigger>
                                            <PopoverContent className="w-96 bg-zinc-900 border-zinc-700 text-zinc-200 shadow-2xl p-4">
                                                <div className="flex items-center gap-2 mb-3 text-red-400">
                                                    <ShieldAlert className="w-4 h-4" />
                                                    <h4 className="font-bold">Format Validation Errors</h4>
                                                </div>
                                                <div className="bg-black/40 rounded-lg p-3 font-mono text-[11px] border border-zinc-800 max-h-60 overflow-y-auto">
                                                    {val?.errors?.length ? (
                                                        <ul className="space-y-2">
                                                            {val.errors.map((error, i) => (
                                                                <li key={i} className="text-red-300/80 leading-relaxed border-b border-zinc-800/50 pb-2 last:border-0">{error}</li>
                                                            ))}
                                                        </ul>
                                                    ) : (
                                                        <p className="text-zinc-500 italic">No specific schema errors reported, but validation failed.</p>
                                                    )}
                                                </div>
                                            </PopoverContent>
                                        </Popover>
                                    )}
                                </TableCell>
                                <TableCell>
                                    {val?.execExists ? (
                                        <div className="flex items-center gap-1.5 text-emerald-400 font-bold bg-emerald-500/10 px-2 py-1 rounded-lg border border-emerald-500/20">
                                            <Check className="w-3.5 h-3.5" /> Found
                                        </div>
                                    ) : (
                                        <Popover>
                                            <PopoverTrigger asChild>
                                                <div className="flex items-center gap-1.5 text-amber-400 font-bold bg-amber-500/10 px-2 py-1 rounded-lg border border-amber-500/20 cursor-pointer hover:bg-amber-500/20 transition-colors">
                                                    <CircleAlert className="w-3.5 h-3.5" /> Missing
                                                </div>
                                            </PopoverTrigger>
                                            <PopoverContent className="w-80 bg-zinc-900 border-zinc-700 text-zinc-200 shadow-2xl p-4">
                                                <div className="flex items-center gap-2 mb-2 text-amber-400">
                                                    <Info className="w-4 h-4" />
                                                    <h4 className="font-bold">Binary Resolution Failure</h4>
                                                </div>
                                                <p className="text-xs text-zinc-400 mb-3">The application executable could not be resolved in the system PATH.</p>
                                                <div className="bg-black/40 rounded-lg p-3 font-mono text-[11px] border border-zinc-800">
                                                    <span className="text-zinc-500">Command:</span>
                                                    <code className="block mt-1 text-amber-200/80">{val?.execCommand || "Unknown"}</code>
                                                </div>
                                                <p className="mt-3 text-[10px] text-zinc-500">
                                                    Hint: Check if the application is installed or if the 'Exec' path in the desktop file is correct.
                                                </p>
                                            </PopoverContent>
                                        </Popover>
                                    )}
                                </TableCell>
                                <TableCell className="text-right space-x-1">
                                    {(file.hasBadPattern || !val?.isValid || file.duplicatePaths?.length > 0) && (
                                        <Button variant="ghost" size="icon" onClick={() => handleAutoFix(file)} title="Magic Fix (Automatic)" className="text-amber-500 hover:text-amber-400 hover:bg-amber-500/10">
                                            <WandSparkles className="w-4 h-4" />
                                        </Button>
                                    )}
                                    <Button variant="ghost" size="icon" onClick={() => setViewFile(file.absolutePath)} title="Inspect Differences">
                                        <Eye className="w-4 h-4" />
                                    </Button>
                                    <Button variant="ghost" size="icon" onClick={() => setEditFile(file)} title="Edit Configuration">
                                        <Edit2 className="w-4 h-4 text-indigo-400" />
                                    </Button>
                                    {file.locationType === "userLocal" && (
                                        <Button variant="destructive" size="icon" onClick={() => handleDelete(file)} title="Delete File">
                                            <Trash2 className="w-4 h-4" />
                                        </Button>
                                    )}
                                </TableCell>
                            </TableRow>
                        );
                    })}
                </TableBody>
            </Table>
            </div>
        );
    };

    return (
        <div className="min-h-screen bg-zinc-950 text-zinc-50 p-6 flex flex-col gap-6 font-sans">
            <header className="flex justify-between items-center bg-zinc-900/80 backdrop-blur-md p-4 rounded-xl border border-zinc-800 shadow-xl relative overflow-hidden">
                <div className="absolute top-0 left-0 w-full h-1 bg-linear-to-r from-indigo-500 via-purple-500 to-emerald-500"></div>
                <div>
                    <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
                        Desktop-Linter-GUI
                    </h1>
                    <p className="text-sm text-zinc-400 mt-1">Wayland-native local configuration manager</p>
                </div>
                <div className="flex gap-2">
                    <Button variant="default" onClick={() => setShowWizard(true)} className="bg-indigo-600 hover:bg-indigo-700 text-white shadow-lg shadow-indigo-500/20">
                        <CopyPlus className="w-4 h-4 mr-2" /> New Template
                    </Button>
                    <Button variant="secondary" onClick={handleSnapshot} className="bg-zinc-800 text-zinc-100 hover:bg-zinc-700 border border-zinc-700">
                        <Camera className="w-4 h-4 mr-2" /> Snapshot
                    </Button>
                </div>
            </header>

            {statusMsg && (
                <Alert className="bg-zinc-900 border-zinc-800 text-zinc-200 shadow-md">
                    <Info className="h-4 w-4 text-blue-400" />
                    <AlertTitle className="text-blue-400">Status Update</AlertTitle>
                    <AlertDescription>{statusMsg}</AlertDescription>
                </Alert>
            )}

            <main className="flex-1 flex flex-col gap-4">
                <Tabs defaultValue="files" className="w-full flex flex-col gap-4">
                    <TabsList className="bg-zinc-900/80 border border-zinc-800 p-1 rounded-lg w-max flex self-start shadow-md">
                        <TabsTrigger value="files" className="data-[state=active]:bg-indigo-600 data-[state=active]:text-white rounded-md transition-all"><LayoutGrid className="w-4 h-4 mr-2"/> Desktop Files</TabsTrigger>
                        <TabsTrigger value="mime" className="data-[state=active]:bg-indigo-600 data-[state=active]:text-white rounded-md transition-all"><Link className="w-4 h-4 mr-2"/> MIME Maps</TabsTrigger>
                        <TabsTrigger value="snapshots" className="data-[state=active]:bg-indigo-600 data-[state=active]:text-white rounded-md transition-all"><Clock className="w-4 h-4 mr-2"/> Snapshots</TabsTrigger>
                    </TabsList>

                    <TabsContent value="files" className="animate-in fade-in duration-300">
                        <div className="bg-zinc-900 border border-zinc-800 rounded-xl shadow-lg p-2">
                            <Tabs defaultValue="all" className="w-full">
                                <div className="px-4 pt-4 flex justify-between items-center">
                                    <div className="flex items-center gap-4">
                                        <TabsList className="bg-zinc-950 border border-zinc-800">
                                            <TabsTrigger value="all" className="data-[state=active]:bg-zinc-800">All Files</TabsTrigger>
                                            <TabsTrigger value="userLocal" className="data-[state=active]:bg-zinc-800">User Local</TabsTrigger>
                                            <TabsTrigger value="system" className="data-[state=active]:bg-zinc-800">System</TabsTrigger>
                                            <TabsTrigger value="duplicates" className="data-[state=active]:bg-orange-900/40 data-[state=active]:text-orange-400">Duplicates</TabsTrigger>
                                            <TabsTrigger value="broken" className="data-[state=active]:bg-red-900/40 data-[state=active]:text-red-400">Broken / Issues</TabsTrigger>
                                        </TabsList>
                                        
                                        <div className="flex gap-2 animate-in slide-in-from-left-2 duration-500">
                                            <Button 
                                                variant="outline" 
                                                size="sm" 
                                                onClick={() => handleFixAll("duplicates")}
                                                className="border-orange-500/30 bg-orange-500/5 text-orange-400 hover:bg-orange-500/20 text-[10px] h-8 font-bold uppercase tracking-wider"
                                            >
                                                <ShieldAlert className="w-3 h-3 mr-1" /> Fix All Duplicates
                                            </Button>
                                            <Button 
                                                variant="outline" 
                                                size="sm" 
                                                onClick={() => handleFixAll("broken")}
                                                className="border-red-500/30 bg-red-500/5 text-red-400 hover:bg-red-500/20 text-[10px] h-8 font-bold uppercase tracking-wider"
                                            >
                                                <Wand2 className="w-3 h-3 mr-1" /> Clean Broken
                                            </Button>
                                        </div>
                                    </div>
                                    
                                    <Button variant="ghost" onClick={refreshData} disabled={loading} className="text-zinc-400 hover:text-white">
                                        <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
                                    </Button>
                                </div>

                                <div className="p-4">
                                    <TabsContent value="all">{renderTable()}</TabsContent>
                                    <TabsContent value="userLocal">{renderTable("userLocal")}</TabsContent>
                                    <TabsContent value="system">{renderTable("system")}</TabsContent>
                                    <TabsContent value="duplicates">{renderTable("duplicates")}</TabsContent>
                                    <TabsContent value="broken">{renderTable("broken")}</TabsContent>
                                </div>
                            </Tabs>
                        </div>
                    </TabsContent>
                    
                    <TabsContent value="mime" className="animate-in fade-in duration-300">
                        <MimeManager />
                    </TabsContent>

                    <TabsContent value="snapshots" className="animate-in fade-in duration-300">
                        <SnapshotViewer onViewDiff={setViewDiff} />
                    </TabsContent>
                </Tabs>
            </main>

            {(viewFile || viewDiff) && (
                <DiffViewer 
                    filePath={viewFile} 
                    diffId={viewDiff} 
                    onClose={() => { setViewFile(null); setViewDiff(null); }} 
                />
            )}
            
            {editFile && <EditorModal file={editFile} onClose={() => setEditFile(null)} onRefresh={refreshData} />}
            
            {showWizard && <WizardModal onClose={() => setShowWizard(false)} onRefresh={refreshData} />}
        </div>
    );
}
