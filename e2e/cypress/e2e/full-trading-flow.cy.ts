/// <reference types="cypress" />

describe('Full Trading Flow E2E Test', () => {
  beforeEach(() => {
    cy.task('cleanDatabase');
    cy.task('seedDatabase');
  });

  it('should complete a full trading cycle from login to trade execution', () => {
    // 1. User Registration and Login
    cy.visit('/');
    cy.get('[data-cy=get-started-btn]').click();
    
    // Register new user
    cy.get('[data-cy=register-link]').click();
    cy.get('[data-cy=email-input]').type('trader@test.com');
    cy.get('[data-cy=password-input]').type('SecurePass123!');
    cy.get('[data-cy=confirm-password-input]').type('SecurePass123!');
    cy.get('[data-cy=register-btn]').click();
    
    // Should redirect to dashboard after registration
    cy.url().should('include', '/dashboard');
    cy.get('[data-cy=welcome-message]').should('contain', 'Welcome to Bot Core');

    // 2. Configure Trading Settings
    cy.get('[data-cy=settings-nav]').click();
    cy.get('[data-cy=trading-tab]').click();
    
    // Enable paper trading
    cy.get('[data-cy=paper-trading-toggle]').click();
    cy.get('[data-cy=initial-balance-input]').clear().type('10000');
    
    // Select trading pairs
    cy.get('[data-cy=trading-pairs-select]').click();
    cy.get('[data-cy=btcusdt-option]').click();
    cy.get('[data-cy=ethusdt-option]').click();
    cy.get('[data-cy=trading-pairs-select]').click(); // Close dropdown
    
    // Set risk parameters
    cy.get('[data-cy=risk-percentage-slider]').invoke('val', 2).trigger('input');
    cy.get('[data-cy=stop-loss-input]').clear().type('2');
    cy.get('[data-cy=take-profit-input]').clear().type('4');
    
    cy.get('[data-cy=save-settings-btn]').click();
    cy.get('[data-cy=success-toast]').should('contain', 'Settings saved');

    // 3. Navigate to Trading Dashboard
    cy.get('[data-cy=dashboard-nav]').click();
    
    // Verify WebSocket connection
    cy.window().its('WebSocketConnected').should('be.true');
    
    // Check market data is loading
    cy.get('[data-cy=market-data-panel]', { timeout: 15000 }).should('be.visible');
    cy.get('[data-cy=btc-price]').should('not.be.empty');
    cy.get('[data-cy=eth-price]').should('not.be.empty');

    // 4. Monitor AI Signals
    cy.get('[data-cy=ai-signals-panel]').should('be.visible');
    
    // Wait for AI analysis
    cy.get('[data-cy=ai-signal-item]', { timeout: 30000 }).should('have.length.greaterThan', 0);
    
    // Verify signal details
    cy.get('[data-cy=ai-signal-item]').first().within(() => {
      cy.get('[data-cy=signal-symbol]').should('not.be.empty');
      cy.get('[data-cy=signal-type]').should('match', /BUY|SELL|HOLD/);
      cy.get('[data-cy=signal-confidence]').should('exist');
    });

    // 5. Execute Paper Trade
    cy.get('[data-cy=paper-trading-nav]').click();
    
    // Wait for trading interface
    cy.get('[data-cy=trading-interface]').should('be.visible');
    
    // Place a trade based on AI signal
    cy.get('[data-cy=ai-suggested-trades]').should('be.visible');
    cy.get('[data-cy=execute-suggested-trade-btn]').first().click();
    
    // Confirm trade dialog
    cy.get('[data-cy=confirm-trade-dialog]').should('be.visible');
    cy.get('[data-cy=confirm-trade-btn]').click();
    
    // Verify trade execution
    cy.get('[data-cy=success-toast]').should('contain', 'Trade executed successfully');
    
    // Check position in portfolio
    cy.get('[data-cy=portfolio-panel]').should('be.visible');
    cy.get('[data-cy=position-item]').should('have.length.greaterThan', 0);

    // 6. Monitor Trade Performance
    cy.get('[data-cy=performance-chart]').should('be.visible');
    cy.get('[data-cy=total-pnl]').should('exist');
    cy.get('[data-cy=win-rate]').should('exist');
    
    // 7. Check Transaction History
    cy.get('[data-cy=history-tab]').click();
    cy.get('[data-cy=transaction-table]').should('be.visible');
    cy.get('[data-cy=transaction-row]').should('have.length.greaterThan', 0);
    
    // Verify transaction details
    cy.get('[data-cy=transaction-row]').first().within(() => {
      cy.get('[data-cy=tx-symbol]').should('not.be.empty');
      cy.get('[data-cy=tx-type]').should('not.be.empty');
      cy.get('[data-cy=tx-amount]').should('not.be.empty');
      cy.get('[data-cy=tx-status]').should('contain', 'Completed');
    });

    // 8. Export Performance Report
    cy.get('[data-cy=export-report-btn]').click();
    cy.get('[data-cy=export-format-select]').select('PDF');
    cy.get('[data-cy=confirm-export-btn]').click();
    
    // Verify download
    cy.readFile('cypress/downloads/trading-report.pdf').should('exist');

    // 9. Test Error Scenarios
    // Simulate network error
    cy.intercept('GET', '/api/market-data/*', { forceNetworkError: true }).as('networkError');
    cy.reload();
    cy.get('[data-cy=error-message]').should('contain', 'Connection error');
    
    // 10. Logout
    cy.get('[data-cy=user-menu]').click();
    cy.get('[data-cy=logout-btn]').click();
    cy.url().should('equal', Cypress.config().baseUrl + '/');
  });

  it('should handle real-time updates correctly', () => {
    cy.login('trader@test.com', 'SecurePass123!');
    cy.visit('/dashboard');
    
    // Setup WebSocket spy
    cy.window().its('WebSocket').then((ws) => {
      cy.spy(ws, 'send').as('wsSend');
      cy.spy(ws, 'onmessage').as('wsReceive');
    });
    
    // Wait for real-time updates
    cy.wait(5000);
    
    // Verify WebSocket messages
    cy.get('@wsReceive').should('have.been.called');
    
    // Check UI updates with real-time data
    cy.get('[data-cy=btc-price]').then(($price1) => {
      const price1 = $price1.text();
      cy.wait(10000); // Wait for price update
      cy.get('[data-cy=btc-price]').then(($price2) => {
        const price2 = $price2.text();
        expect(price1).not.to.equal(price2); // Price should have changed
      });
    });
  });

  it('should enforce trading limits and risk management', () => {
    cy.login('trader@test.com', 'SecurePass123!');
    cy.visit('/paper-trading');
    
    // Try to place trade exceeding balance
    cy.get('[data-cy=manual-trade-btn]').click();
    cy.get('[data-cy=trade-amount-input]').type('15000'); // More than balance
    cy.get('[data-cy=place-trade-btn]').click();
    
    // Should show error
    cy.get('[data-cy=error-toast]').should('contain', 'Insufficient balance');
    
    // Try to place trade exceeding risk limit
    cy.get('[data-cy=trade-amount-input]').clear().type('5000'); // 50% of balance
    cy.get('[data-cy=place-trade-btn]').click();
    
    // Should show risk warning
    cy.get('[data-cy=risk-warning-dialog]').should('be.visible');
    cy.get('[data-cy=cancel-trade-btn]').click();
  });
});