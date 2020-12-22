Feature: Working with passwords
  Scenario: Opening a password and reading the passphrase
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    Then the passphrase can be read

  Scenario: Opening a password and reading a comment
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    Then a comment can be read

  Scenario: Opening a password and reading an entry
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    Then an entry can be read

  Scenario: Opening a password and creating a QR code for passphrase
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    Then a QR code can be created for the passphrase

  Scenario: Opening a password and creating a QR code for comment
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    Then a QR code can be created for a comment

  Scenario: Opening a password and creating a QR code for entry
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    Then a QR code can be created for an entry

  Scenario: Opening a password and creating a QR code for whole password file
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    Then a QR code can be created for the complete password

  Scenario: Create a QR code for encrypted password file
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    Then a QR code can be created for an encrypted password

  Scenario: Creating a password
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a new password is created
    Then the new password appears in the password store

  Scenario: Generating a passphrase and creating it
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a new password is created with a generated passphrase
    Then a new password is created

  Scenario: Editing a password by generating a new passphrase
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And the passphrase of the password is set to a newly created one
    Then the password is modified

  Scenario: Editing a password by editing an entry
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And an entry of the password is changed
    Then the password is modified

  Scenario: Editing a password by editing a comment
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And a comment of the password is changed
    Then the password is modified

  Scenario: Editing a password by adding an entry
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And an entry is added to the password
    Then the password is modified

  Scenario: Editing a password by adding a comment
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And a comment is added to the password
    Then the password is modified

  Scenario: Editing a password by inserting an entry
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And an entry is inserted in the password
    Then the password is modified

  Scenario: Editing a password by inserting a comment
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And a comment is inserted in the password
    Then the password is modified

  Scenario: Editing a password by removing an entry
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And an entry is removed from the password
    Then the password is modified

  Scenario: Editing a password by removing a comment
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    And a comment is removed from the password
    Then the password is modified

  Scenario: Removing a password
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is removed
    Then the password does not exist

  Scenario: Renaming a password
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is renamed
    Then the password has a different name

  Scenario: Copying a password
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is copied
    Then a new password exists
