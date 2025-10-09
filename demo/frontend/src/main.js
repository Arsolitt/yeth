// Frontend entry point
import { API_VERSION } from '../../shared/constants.js';
import { formatDate } from '../../shared/utils.js';

console.log('Frontend app started');
console.log('API Version:', API_VERSION);
console.log('Current time:', formatDate(new Date()));

async function fetchData() {
    try {
        const response = await fetch('http://localhost:8080/api/users');
        const data = await response.json();
        console.log('Users:', data);
    } catch (error) {
        console.error('Failed to fetch data:', error);
    }
}

fetchData();

