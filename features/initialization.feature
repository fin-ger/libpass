Feature: Initialize a new password store
  Scenario: A new password store is created at the default location
    Given no password store exists
    And a new password store is initialized
    When a new password store is successfully created
    Then the password store has no errors
    And the password store is empty
    And the password store's directory exists
    And the password store's directory contains a GPG ID file

  Scenario: A new password store is created from the environment
    Given the password store location is set in the environment
    And no password store exists
    And a new password store is initialized
    When a new password store is successfully created
    Then the password store has no errors
    And the password store is empty
    And the password store's directory exists
    And the password store's directory contains a GPG ID file

  Scenario: A new password store is created at a manually provided location
    Given no password store exists
    And a new password store is initialized at a manually provided location
    When a new password store is successfully created
    Then the password store has no errors
    And the password store is empty
    And the password store's directory exists
    And the password store's directory contains a GPG ID file

  Scenario: A new password store fails to create if it already exists
    Given a password store exists
    And a new password store is initialized
    Then the initialization of the password store fails
