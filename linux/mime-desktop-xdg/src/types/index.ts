export type LocationType = 'userLocal' | 'system' | 'systemLocal' | 'flatpak' | 'other';

export interface DesktopFile {
    absolutePath: string;
    filename: string;
    locationType: LocationType;
    isShadowed: boolean;
    shadowsPaths: string[];
    parsedName: string | null;
    parsedExec: string | null;
    hasBadPattern: boolean;
    duplicatePaths: string[];
}

export interface ValidationResult {
    isValid: boolean;
    errors: string[];
    execExists: boolean;
    execCommand: string | null;
}

export interface MimeAssociation {
    mimeType: string;
    defaultApps: string[];
}

export interface GitSnapshot {
    commit_id: string;
    message: string;
    timestamp: number;
    author: string;
}
