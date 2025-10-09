// Shared utilities
export function formatDate(date) {
    return date.toISOString();
}

export function parseJSON(str) {
    try {
        return JSON.parse(str);
    } catch (e) {
        return null;
    }
}

