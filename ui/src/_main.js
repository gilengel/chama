import App from './App.svelte';
//import wasm from '../../rust/Cargo.toml';

const init = async () => {
	//const { Editor } = await wasm();

	new App({
		target: document.body,
		props: {
			name: 'world',
			//Editor: Editor
		}
	});
}

init();