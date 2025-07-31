import { defineConfig } from 'cypress';

export default defineConfig({
  e2e: {
    baseUrl: 'http://localhost:3000',
    viewportWidth: 1920,
    viewportHeight: 1080,
    video: true,
    screenshotOnRunFailure: true,
    chromeWebSecurity: false,
    defaultCommandTimeout: 10000,
    requestTimeout: 10000,
    responseTimeout: 10000,
    
    env: {
      API_URL: 'http://localhost:8080',
      AI_API_URL: 'http://localhost:8000',
      WS_URL: 'ws://localhost:8080/ws',
      TEST_USER_EMAIL: 'test@example.com',
      TEST_USER_PASSWORD: 'Test123!@#',
    },
    
    setupNodeEvents(on, config) {
      // Task for database seeding
      on('task', {
        async seedDatabase() {
          // Database seeding logic
          return null;
        },
        async cleanDatabase() {
          // Database cleanup logic
          return null;
        },
        log(message) {
          console.log(message);
          return null;
        },
      });
      
      // Code coverage
      require('@cypress/code-coverage/task')(on, config);
      
      return config;
    },
  },
  
  component: {
    devServer: {
      framework: 'react',
      bundler: 'vite',
    },
    specPattern: 'src/**/*.cy.{js,jsx,ts,tsx}',
  },
});