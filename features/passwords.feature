Feature: Working with passwords
  Scenario: Opening a password and reading the passphrase
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then the passphrase can be read

  Scenario: Opening a password and reading a comment
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then a comment can be read

  Scenario: Opening a password and reading an entry
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then an entry can be read

  Scenario: Opening a password and creating a QR code for passphrase
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then a QR code can be created for the passphrase

  Scenario: Opening a password and creating a QR code for comment
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then a QR code can be created for a comment

  Scenario: Opening a password and creating a QR code for entry
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then a QR code can be created for an entry

  Scenario: Opening a password and creating a QR code for whole password file
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then a QR code can be created for the complete password

  Scenario: Create a QR code for encrypted password file
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    Then a QR code can be created for an encrypted password

  Scenario: Creating a password
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    Then a new password can be created

  Scenario: Generating a passphrase and creating it
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    Then a new password can be created with a generated passphrase

  Scenario: Editing a password by generating a new passphrase
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then the passphrase can be set to a newly generated one

  Scenario: Editing a password by editing an entry
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then an entry can be changed

  Scenario: Editing a password by editing a comment
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then a comment can be changed

  Scenario: Editing a password by adding an entry
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then an entry can be added

  Scenario: Editing a password by adding a comment
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then a comment can be added

  Scenario: Editing a password by inserting an entry
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then an entry can be inserted

  Scenario: Editing a password by inserting a comment
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then a comment can be inserted

  Scenario: Editing a password by removing an entry
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then an entry can be removed

  Scenario: Editing a password by removing a comment
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened
    Then a comment can be removed

  Scenario: Removing a password
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    Then a password can be removed

  Scenario: Renaming a password
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    Then a password can be renamed
