import App from './App.svelte';

const init = async () => {
	new App({
		target: document.body,
	});
}

init();