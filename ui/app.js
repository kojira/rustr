import init, { start_app } from './ui.js';

async function run() {
    try {
        // WASM初期化
        await init();
        
        // アプリケーション起動
        await start_app('canvas');
        
        // ローディング画面を非表示
        document.getElementById('loading').style.display = 'none';
        
        console.log('Rustr started successfully');
    } catch (error) {
        console.error('Failed to start Rustr:', error);
        document.getElementById('loading').innerHTML = `
            <h1>❌ Error</h1>
            <p>Failed to start application</p>
            <p style="color: #ff4444; margin-top: 20px;">${error.message}</p>
        `;
    }
}

run();

