import QtQuick 2.15
import org.kde.plasma.components 3.0 as PlasmaComponents
import org.kde.plasma.core 2.1 as PlasmaCore

Item {
    id: compact
    
    PlasmaComponents.Label {
        anchors.centerIn: parent
        text: {
            if (root.connectionStatus === "connected") {
                if (root.hasLeftRight) {
                    return `${root.batteryLeft}% | ${root.batteryRight}%`
                } else {
                    return `${root.batteryLevel}%`
                }
            } else {
                return "---"
            }
        }
        font.pixelSize: Math.max(theme.smallestFont.pixelSize, parent.height * 0.5)
        color: {
            if (root.connectionStatus === "disconnected") {
                return theme.disabledTextColor
            } else if (root.hasLeftRight) {
                var avg = (root.batteryLeft + root.batteryRight) / 2
                if (avg < 20) return theme.negativeTextColor
                if (avg < 50) return theme.neutralTextColor
                return theme.positiveTextColor
            } else {
                if (root.batteryLevel < 20) return theme.negativeTextColor
                if (root.batteryLevel < 50) return theme.neutralTextColor
                return theme.positiveTextColor
            }
        }
    }
    
    PlasmaCore.IconItem {
        anchors {
            right: parent.right
            rightMargin: 2
            verticalCenter: parent.verticalCenter
        }
        width: parent.height * 0.6
        height: width
        source: {
            if (root.connectionStatus === "disconnected") {
                return "audio-headphones"
            } else if (root.isCharging || root.isChargingLeft || root.isChargingRight) {
                return "battery-charging"
            } else {
                return "battery"
            }
        }
        visible: parent.width > theme.mSize(theme.defaultFont).width * 3
    }
}

