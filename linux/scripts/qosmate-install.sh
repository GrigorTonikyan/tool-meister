#!/bin/sh

# backend install
LATEST_TAG=$(uclient-fetch -O - https://api.github.com/repos/hudra0/qosmate/releases/latest 2>/dev/null | grep -o '"tag_name":"[^"]*' | sed 's/"tag_name":"//') &&
    uclient-fetch -O /etc/init.d/qosmate https://raw.githubusercontent.com/hudra0/qosmate/$LATEST_TAG/etc/init.d/qosmate && chmod +x /etc/init.d/qosmate &&
    uclient-fetch -O /etc/qosmate.sh https://raw.githubusercontent.com/hudra0/qosmate/$LATEST_TAG/etc/qosmate.sh && chmod +x /etc/qosmate.sh &&
    [ ! -f /etc/config/qosmate ] && uclient-fetch -O /etc/config/qosmate https://raw.githubusercontent.com/hudra0/qosmate/$LATEST_TAG/etc/config/qosmate
/etc/init.d/qosmate enable

# frontend install
LATEST_TAG=$(uclient-fetch -O - https://api.github.com/repos/hudra0/luci-app-qosmate/releases/latest 2>/dev/null | grep -o '"tag_name":"[^"]*' | sed 's/"tag_name":"//') &&
    mkdir -p /www/luci-static/resources/view/qosmate /usr/share/luci/menu.d /usr/share/rpcd/acl.d /usr/libexec/rpcd &&
    uclient-fetch -O /www/luci-static/resources/view/qosmate/settings.js https://raw.githubusercontent.com/hudra0/luci-app-qosmate/$LATEST_TAG/htdocs/luci-static/resources/view/settings.js &&
    uclient-fetch -O /www/luci-static/resources/view/qosmate/hfsc.js https://raw.githubusercontent.com/hudra0/luci-app-qosmate/$LATEST_TAG/htdocs/luci-static/resources/view/hfsc.js &&
    uclient-fetch -O /www/luci-static/resources/view/qosmate/cake.js https://raw.githubusercontent.com/hudra0/luci-app-qosmate/$LATEST_TAG/htdocs/luci-static/resources/view/cake.js &&
    uclient-fetch -O /www/luci-static/resources/view/qosmate/advanced.js https://raw.githubusercontent.com/hudra0/luci-app-qosmate/$LATEST_TAG/htdocs/luci-static/resources/view/advanced.js &&
    uclient-fetch -O /www/luci-static/resources/view/qosmate/rules.js https://raw.githubusercontent.com/hudra0/luci-app-qosmate/$LATEST_TAG/htdocs/luci-static/resources/view/rules.js &&
    uclient-fetch -O /www/luci-static/resources/view/qosmate/connections.js https://raw.githubusercontent.com/hudra0/luci-app-qosmate/$LATEST_TAG/htdocs/luci-static/resources/view/connections.js &&
    uclient-fetch -O /www/luci-static/resources/view/qosmate/custom_rules.js https://raw.githubusercontent.com/hudra0/luci-app-qosmate/$LATEST_TAG/htdocs/luci-static/resources/view/custom_rules.js &&
    uclient-fetch -O /www/luci-static/resources/view/qosmate/ipsets.js https://raw.githubusercontent.com/hudra0/luci-app-qosmate/$LATEST_TAG/htdocs/luci-static/resources/view/ipsets.js &&
    uclient-fetch -O /www/luci-static/resources/view/qosmate/statistics.js https://raw.githubusercontent.com/hudra0/luci-app-qosmate/$LATEST_TAG/htdocs/luci-static/resources/view/statistics.js &&
    uclient-fetch -O /usr/share/luci/menu.d/luci-app-qosmate.json https://raw.githubusercontent.com/hudra0/luci-app-qosmate/$LATEST_TAG/root/usr/share/luci/menu.d/luci-app-qosmate.json &&
    uclient-fetch -O /usr/share/rpcd/acl.d/luci-app-qosmate.json https://raw.githubusercontent.com/hudra0/luci-app-qosmate/$LATEST_TAG/root/usr/share/rpcd/acl.d/luci-app-qosmate.json &&
    uclient-fetch -O /usr/libexec/rpcd/luci.qosmate https://raw.githubusercontent.com/hudra0/luci-app-qosmate/$LATEST_TAG/root/usr/libexec/rpcd/luci.qosmate &&
    uclient-fetch -O /usr/libexec/rpcd/luci.qosmate_stats https://raw.githubusercontent.com/hudra0/luci-app-qosmate/$LATEST_TAG/root/usr/libexec/rpcd/luci.qosmate_stats
chmod +x /usr/libexec/rpcd/luci.qosmate
chmod +x /usr/libexec/rpcd/luci.qosmate_stats
sleep 1
/etc/init.d/rpcd restart
sleep 1
/etc/init.d/uhttpd restart

sleep 1
/etc/init.d/qosmate start
