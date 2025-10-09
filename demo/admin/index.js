// Admin panel
import { formatDate } from '../shared/utils.js';

console.log('Admin panel initialized');

class AdminPanel {
    constructor() {
        this.users = [];
        this.logs = [];
    }

    async loadUsers() {
        console.log('Loading users...');
        // Fetch users from backend
    }

    async viewLogs() {
        console.log('Viewing logs at', formatDate(new Date()));
        // Fetch logs from backend
    }

    async manageSettings() {
        console.log('Managing settings...');
        // Settings management
    }
}

const admin = new AdminPanel();
admin.loadUsers();

