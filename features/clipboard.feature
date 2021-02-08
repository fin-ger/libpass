Feature: Place password contents in the clipboard
  Scenario: Placing a password in the clipboard for an automatically detected duration
    Given a password store exists
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And the passphrase is copied for an automatically detected duration to the clipboard
    Then the passphrase lasts in the clipboard for 45 seconds

  Scenario: Placing a password in the clipboard for a manually provided location
    Given a password store exists
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And the passphrase is copied for a manually provided duration of 1 second to the clipboard
    Then the passphrase lasts in the clipboard for 1 second

  Scenario: Placing a password in the clipboard for a duration specified by the environment
    Given a password store exists
    And passwords are stored in the password store
    And a clipboard duration of 1 second is specified in the environment
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And the passphrase is copied for an automatically detected duration to the clipboard
    Then the passphrase lasts in the clipboard for 1 second

  Scenario: Placing a password in the clipboard with respect to an automatic X selection
    Given a password store exists
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And the passphrase is copied to an automatically detected X selection
    Then the passphrase is copied to the clipboard X selection

  Scenario: Placing a password in the clipboard with respect to the clipboard X selection
    Given a password store exists
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And the passphrase is copied to the clipboard X selection
    Then the passphrase is copied to the clipboard X selection

  Scenario: Placing a password in the clipboard with respect to the primary X selection
    Given a password store exists
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And the passphrase is copied to the primary X selection
    Then the passphrase is copied to the primary X selection

  Scenario: Placing a password in the clipboard with respect to the secondary X selection
    Given a password store exists
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And the passphrase is copied to the secondary X selection
    Then the passphrase is copied to the secondary X selection

  Scenario: Placing a password in the clipboard with respect to the clipboard X selection from the environment
    Given a password store exists
    And passwords are stored in the password store
    And the X selection is set to clipboard in the environment
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And the passphrase is copied to an automatically detected X selection
    Then the passphrase is copied to the clipboard X selection

  Scenario: Placing a password in the clipboard with respect to the primary X selection from the environment
    Given a password store exists
    And passwords are stored in the password store
    And the X selection is set to primary in the environment
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And the passphrase is copied to an automatically detected X selection
    Then the passphrase is copied to the primary X selection

  Scenario: Placing a password in the clipboard with respect to the secondary X selection from the environment
    Given a password store exists
    And passwords are stored in the password store
    And the X selection is set to secondary in the environment
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And the passphrase is copied to an automatically detected X selection
    Then the passphrase is copied to the secondary X selection
