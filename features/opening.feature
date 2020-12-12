Feature: Open an existing password store
  Scenario: Open an existing password store at the default location
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    Then the password store has no errors
    And the password store contains passwords

  Scenario: Open a non-existent password store at the default location
    Given no password store exists
    And a password store is opened at the default location
    Then the opening of the password store fails

  Scenario: Open an existing password store from the environment variable
    Given a password store exists from the environment variable
    And a password store is opened from the environment variable
    When the password store is successfully opened
    Then the password store has no errors
    And the password store contains passwords

  Scenario: Open a non-existent password store from the environment variable
    Given no password store exists
    And a password store is opened from the environment variable
    Then the opening of the password store fails

  Scenario: Open an existing password store at a manually provided location
    Given a password store exists at a manually provided location
    And a password store is opened at a manually provided location
    When the password store is successfully opened
    Then the password store has no errors
    And the password store contains passwords

  Scenario: Open a non-existent password store at a manually provided location
    Given no password store exists
    And a password store is opened at a manually provided location
    Then the opening of the password store fails
