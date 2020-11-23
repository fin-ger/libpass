Feature: Initialize a new password store
  Scenario: A new password store is created
    Given no password store exists
    When a new password store is initialized
    And a new password store is successfully created
    Then the password store has no errors
    And the password store is empty
    And the password store's directory exists
    And the password store's directory contains a GPG ID file

  Scenario: A new password store fails to create if it already exists
    Given a password store exists at the default location
    When a new password store is initialized
    And the initialization of the password store fails
    Then the password store's directory does not exist
