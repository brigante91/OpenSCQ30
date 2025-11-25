import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents
import org.kde.plasma.core 2.1 as PlasmaCore
import org.kde.kirigami 2.20 as Kirigami

ColumnLayout {
    id: fullView
    spacing: Kirigami.Units.smallSpacing
    
    // Device Name
    PlasmaComponents.Label {
        Layout.fillWidth: true
        text: root.deviceName
        font.bold: true
        font.pixelSize: theme.defaultFont.pixelSize * 1.2
        horizontalAlignment: Text.AlignHCenter
        elide: Text.ElideRight
    }
    
    // Connection Status
    PlasmaComponents.Label {
        Layout.fillWidth: true
        text: root.connectionStatus === "connected" ? "Connected" : "Not Connected"
        font.pixelSize: theme.smallFont.pixelSize
        color: root.connectionStatus === "connected" ? theme.positiveTextColor : theme.negativeTextColor
        horizontalAlignment: Text.AlignHCenter
    }
    
    // Separator
    PlasmaCore.Separator {
        Layout.fillWidth: true
        Layout.topMargin: Kirigami.Units.smallSpacing
        Layout.bottomMargin: Kirigami.Units.smallSpacing
    }
    
    // Battery Display
    ColumnLayout {
        Layout.fillWidth: true
        visible: root.connectionStatus === "connected"
        spacing: Kirigami.Units.smallSpacing
        
        // Dual Battery (Left/Right)
        RowLayout {
            Layout.fillWidth: true
            visible: root.hasLeftRight
            
            ColumnLayout {
                Layout.fillWidth: true
                
                PlasmaComponents.Label {
                    text: "Left"
                    font.pixelSize: theme.smallFont.pixelSize
                }
                
                PlasmaComponents.ProgressBar {
                    Layout.fillWidth: true
                    value: root.batteryLeft
                    maximumValue: 100
                }
                
                PlasmaComponents.Label {
                    text: `${root.batteryLeft}%${root.isChargingLeft ? " ⚡" : ""}`
                    font.pixelSize: theme.smallFont.pixelSize
                    color: {
                        if (root.batteryLeft < 20) return theme.negativeTextColor
                        if (root.batteryLeft < 50) return theme.neutralTextColor
                        return theme.positiveTextColor
                    }
                }
            }
            
            ColumnLayout {
                Layout.fillWidth: true
                
                PlasmaComponents.Label {
                    text: "Right"
                    font.pixelSize: theme.smallFont.pixelSize
                }
                
                PlasmaComponents.ProgressBar {
                    Layout.fillWidth: true
                    value: root.batteryRight
                    maximumValue: 100
                }
                
                PlasmaComponents.Label {
                    text: `${root.batteryRight}%${root.isChargingRight ? " ⚡" : ""}`
                    font.pixelSize: theme.smallFont.pixelSize
                    color: {
                        if (root.batteryRight < 20) return theme.negativeTextColor
                        if (root.batteryRight < 50) return theme.neutralTextColor
                        return theme.positiveTextColor
                    }
                }
            }
        }
        
        // Single Battery
        ColumnLayout {
            Layout.fillWidth: true
            visible: !root.hasLeftRight
            
            PlasmaComponents.Label {
                text: "Battery"
                font.pixelSize: theme.smallFont.pixelSize
            }
            
            PlasmaComponents.ProgressBar {
                Layout.fillWidth: true
                value: root.batteryLevel
                maximumValue: 100
            }
            
            PlasmaComponents.Label {
                text: `${root.batteryLevel}%${root.isCharging ? " ⚡" : ""}`
                font.pixelSize: theme.smallFont.pixelSize
                color: {
                    if (root.batteryLevel < 20) return theme.negativeTextColor
                    if (root.batteryLevel < 50) return theme.neutralTextColor
                    return theme.positiveTextColor
                }
            }
        }
        
        // Case Battery
        ColumnLayout {
            Layout.fillWidth: true
            visible: root.caseBattery >= 0
            
            PlasmaComponents.Label {
                text: "Case"
                font.pixelSize: theme.smallFont.pixelSize
            }
            
            PlasmaComponents.ProgressBar {
                Layout.fillWidth: true
                value: root.caseBattery
                maximumValue: 100
            }
            
            PlasmaComponents.Label {
                text: `${root.caseBattery}%`
                font.pixelSize: theme.smallFont.pixelSize
                color: {
                    if (root.caseBattery < 20) return theme.negativeTextColor
                    if (root.caseBattery < 50) return theme.neutralTextColor
                    return theme.positiveTextColor
                }
            }
        }
    }
    
    // Not Connected Message
    PlasmaComponents.Label {
        Layout.fillWidth: true
        Layout.topMargin: Kirigami.Units.largeSpacing
        visible: root.connectionStatus === "disconnected"
        text: "Connect a device using OpenSCQ30 GUI or CLI"
        font.pixelSize: theme.smallFont.pixelSize
        color: theme.disabledTextColor
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
    }
}

